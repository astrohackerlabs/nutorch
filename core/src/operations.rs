//! Native operation evaluation. Parameters contain values and owned tensor
//! references; there is no wire request, registry lookup, or evaluator callback.
use crate::{Data, Device, Kind, Scalar, SharedTensor, Tensor, mark_requires_grad};
use nutorch_ops::{Arity, OpSpec, ParamKind};
use std::collections::BTreeMap;

type OpError = crate::Error;

#[derive(Clone, Debug)]
pub enum Parameter {
    Int(i64),
    Float(f64),
    IntList(Vec<i64>),
    Bool(bool),
    Str(String),
    Tensor(SharedTensor),
}

#[derive(Clone, Debug, Default)]
pub struct Params {
    pub values: BTreeMap<String, Parameter>,
}

impl Params {
    pub fn validate(&self, spec: &OpSpec) -> crate::Result<()> {
        for key in self.values.keys() {
            if !spec.params.iter().any(|p| p.name == key) {
                return Err(crate::argument(format!(
                    "{}: unknown parameter: {key}",
                    spec.name
                )));
            }
        }
        for p in spec.params {
            let Some(value) = self.values.get(p.name) else {
                if p.required {
                    return Err(crate::argument(format!(
                        "{}: missing required parameter: {}",
                        spec.name, p.name
                    )));
                }
                continue;
            };
            let valid = matches!(
                (p.kind, value),
                (ParamKind::Int, Parameter::Int(_))
                    | (
                        ParamKind::Float | ParamKind::Scalar | ParamKind::TensorOrScalar,
                        Parameter::Int(_) | Parameter::Float(_)
                    )
                    | (ParamKind::IntList, Parameter::IntList(_))
                    | (ParamKind::Bool, Parameter::Bool(_))
                    | (ParamKind::Str, Parameter::Str(_))
                    | (ParamKind::TensorOrScalar, Parameter::Tensor(_))
            );
            if !valid {
                return Err(crate::argument(format!(
                    "{}: parameter {} must be {:?}",
                    spec.name, p.name, p.kind
                )));
            }
        }
        Ok(())
    }
    pub(crate) fn int(&self, name: &str) -> Option<i64> {
        match self.values.get(name)? {
            Parameter::Int(i) => Some(*i),
            _ => None,
        }
    }
    pub(crate) fn float(&self, name: &str) -> Option<f64> {
        match self.values.get(name)? {
            Parameter::Int(i) => Some(*i as f64),
            Parameter::Float(f) => Some(*f),
            _ => None,
        }
    }
    fn scalar(&self, name: &str) -> Option<Scalar> {
        match self.values.get(name)? {
            Parameter::Int(i) => Some(Scalar::Int(*i)),
            Parameter::Float(f) => Some(Scalar::Float(*f)),
            _ => None,
        }
    }
    pub(crate) fn int_list(&self, name: &str) -> Option<Vec<i64>> {
        match self.values.get(name)? {
            Parameter::IntList(v) => Some(v.clone()),
            _ => None,
        }
    }
    pub(crate) fn bool(&self, name: &str) -> bool {
        matches!(self.values.get(name), Some(Parameter::Bool(true)))
    }
    fn str(&self, name: &str) -> Option<&str> {
        match self.values.get(name)? {
            Parameter::Str(s) => Some(s),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum Output {
    Tensors(Vec<SharedTensor>),
    Value(Data),
    Nothing,
}
enum Applied {
    Tensors(Vec<Tensor>),
    Value(Data),
    Nothing,
}

pub fn execute(spec: &OpSpec, inputs: &[SharedTensor], params: &Params) -> crate::Result<Output> {
    let valid = match spec.tensors {
        Arity::Exactly(n) => inputs.len() == n,
        Arity::AtLeast(n) => inputs.len() >= n,
    };
    if !valid {
        return Err(crate::argument(format!(
            "{}: expected {:?} tensor(s), got {}",
            spec.name,
            spec.tensors,
            inputs.len()
        )));
    }
    params.validate(spec)?;
    let guard = crate::gate()?;
    if !tch::utils::has_mps() {
        return Err(crate::argument(
            "NuTorch requires an Apple-silicon Mac with MPS (GPU-only by design)",
        ));
    }
    let tensors = inputs
        .iter()
        .map(|t| t.alias_under_gate(&guard))
        .collect::<crate::Result<Vec<_>>>()?;
    let param_tensors = params
        .values
        .iter()
        .filter_map(|(name, value)| match value {
            Parameter::Tensor(t) => Some(t.alias_under_gate(&guard).map(|t| (name.as_str(), t))),
            _ => None,
        })
        .collect::<crate::Result<Vec<_>>>()?;
    if spec.broadcasts
        && tensors.len() == 2
        && !broadcastable(&tensors[0].size(), &tensors[1].size())
    {
        return Err((
            "shape_mismatch",
            format!(
                "{}: shapes {:?} and {:?} are not broadcastable",
                spec.name,
                tensors[0].size(),
                tensors[1].size()
            ),
        ));
    }
    let refs = tensors.iter().collect::<Vec<_>>();
    let param_refs = param_tensors.iter().map(|(name, t)| (*name, t)).collect();
    Ok(match apply(spec, &refs, params, &param_refs)? {
        Applied::Tensors(ts) => Output::Tensors(ts.into_iter().map(SharedTensor::new).collect()),
        Applied::Value(v) => Output::Value(v),
        Applied::Nothing => Output::Nothing,
    })
}

fn tch(op: &str, e: tch::TchError) -> OpError {
    ("torch_error", format!("{op}: {}", crate::tch_error(e)))
}
fn parse_reduction(p: &Params) -> Result<tch::Reduction, OpError> {
    match p.str("reduction") {
        None | Some("mean") => Ok(tch::Reduction::Mean),
        Some("sum") => Ok(tch::Reduction::Sum),
        Some("none") => Ok(tch::Reduction::None),
        Some(other) => Err(crate::argument(format!(
            "invalid reduction: {other} (expected mean, sum, or none)"
        ))),
    }
}
fn broadcastable(a: &[i64], b: &[i64]) -> bool {
    a.iter()
        .rev()
        .zip(b.iter().rev())
        .all(|(a, b)| a == b || *a == 1 || *b == 1)
}

fn one(op: &str, result: Result<Tensor, tch::TchError>) -> Result<Applied, OpError> {
    result
        .map(|t| Applied::Tensors(vec![t]))
        .map_err(|e| tch(op, e))
}

fn apply(
    spec: &OpSpec,
    t: &[&Tensor],
    p: &Params,
    pt: &std::collections::HashMap<&str, &Tensor>,
) -> Result<Applied, OpError> {
    let op = spec.name;
    match op {
        "add" => crate::add(
            t[0],
            t[1],
            p.scalar("alpha").map(|s| match s {
                Scalar::Int(i) => crate::Scalar::Int(i),
                Scalar::Float(f) => crate::Scalar::Float(f),
            }),
        )
        .map(|t| Applied::Tensors(vec![t])),
        // Sub remains legacy: a - alpha*b.
        "sub" => {
            let result = match p.scalar("alpha") {
                None | Some(Scalar::Int(1)) => t[0].f_sub(t[1]),
                Some(alpha) => {
                    let scaled = match alpha {
                        Scalar::Int(i) => t[1].f_mul_scalar(i),
                        Scalar::Float(f) => t[1].f_mul_scalar(f),
                    }
                    .map_err(|e| tch(op, e))?;
                    t[0].f_sub(&scaled)
                }
            };
            one(op, result)
        }
        "sin" => one(op, t[0].f_sin()),
        // --- pointwise sweep (issue 0005 exp 2): unary ---
        "abs" => one(op, t[0].f_abs()),
        "acos" => one(op, t[0].f_acos()),
        "acosh" => one(op, t[0].f_acosh()),
        "asin" => one(op, t[0].f_asin()),
        "asinh" => one(op, t[0].f_asinh()),
        "atan" => one(op, t[0].f_atan()),
        "atanh" => one(op, t[0].f_atanh()),
        "ceil" => one(op, t[0].f_ceil()),
        "cos" => one(op, t[0].f_cos()),
        "cosh" => one(op, t[0].f_cosh()),
        "deg2rad" => one(op, t[0].f_deg2rad()),
        "digamma" => one(op, t[0].f_digamma()),
        "erf" => one(op, t[0].f_erf()),
        "erfc" => one(op, t[0].f_erfc()),
        "exp" => one(op, t[0].f_exp()),
        "exp2" => one(op, t[0].f_exp2()),
        "expm1" => one(op, t[0].f_expm1()),
        "floor" => one(op, t[0].f_floor()),
        "frac" => one(op, t[0].f_frac()),
        "i0" => one(op, t[0].f_i0()),
        "lgamma" => one(op, t[0].f_lgamma()),
        "log" => one(op, t[0].f_log()),
        "log10" => one(op, t[0].f_log10()),
        "log1p" => one(op, t[0].f_log1p()),
        "log2" => one(op, t[0].f_log2()),
        "logit" => one(op, t[0].f_logit(None::<f64>)),
        "neg" => one(op, t[0].f_neg()),
        "rad2deg" => one(op, t[0].f_rad2deg()),
        "reciprocal" => one(op, t[0].f_reciprocal()),
        "relu" => one(op, t[0].f_relu()),
        "round" => one(op, t[0].f_round()),
        "rsqrt" => one(op, t[0].f_rsqrt()),
        "sgn" => one(op, t[0].f_sgn()),
        "sigmoid" => one(op, t[0].f_sigmoid()),
        "sign" => one(op, t[0].f_sign()),
        "sinc" => one(op, t[0].f_sinc()),
        "sinh" => one(op, t[0].f_sinh()),
        "sqrt" => one(op, t[0].f_sqrt()),
        "square" => one(op, t[0].f_square()),
        "tan" => one(op, t[0].f_tan()),
        "tanh" => one(op, t[0].f_tanh()),
        "trunc" => one(op, t[0].f_trunc()),
        "softmax" => one(
            op,
            t[0].f_softmax(p.int("dim").expect("required"), Kind::Float),
        ),
        "log_softmax" => one(
            op,
            t[0].f_log_softmax(p.int("dim").expect("required"), Kind::Float),
        ),
        "nan_to_num" => one(
            op,
            t[0].f_nan_to_num(p.float("nan"), p.float("posinf"), p.float("neginf")),
        ),
        // --- pointwise sweep: binary, broadcasting ---
        "mul" => crate::mul(t[0], t[1]).map(|t| Applied::Tensors(vec![t])),
        "div" => one(op, t[0].f_div(t[1])),
        "maximum" => one(op, t[0].f_maximum(t[1])),
        "minimum" => one(op, t[0].f_minimum(t[1])),
        "atan2" => one(op, t[0].f_atan2(t[1])),
        "fmod" => one(op, t[0].f_fmod_tensor(t[1])),
        "remainder" => one(op, t[0].f_remainder_tensor(t[1])),
        "floor_divide" => one(op, t[0].f_floor_divide(t[1])),
        "hypot" => one(op, t[0].f_hypot(t[1])),
        "copysign" => one(op, t[0].f_copysign(t[1])),
        "xlogy" => one(op, t[0].f_xlogy(t[1])),
        "logaddexp" => one(op, t[0].f_logaddexp(t[1])),
        "pow" => match pt.get("exponent") {
            Some(exponent) => one(op, t[0].f_pow(exponent)),
            None => match p.scalar("exponent").expect("required") {
                Scalar::Int(i) => one(op, t[0].f_pow_tensor_scalar(i)),
                Scalar::Float(f) => one(op, t[0].f_pow_tensor_scalar(f)),
            },
        },
        "clamp" => {
            let to_scalar = |s: Scalar| -> tch::Scalar {
                match s {
                    Scalar::Int(i) => i.into(),
                    Scalar::Float(f) => f.into(),
                }
            };
            // Tensor bounds (TensorOrScalar) take f_clamp_tensor; scalar
            // bounds keep the original single/double-bound calls.
            let (min_t, max_t) = (pt.get("min"), pt.get("max"));
            if min_t.is_some() || max_t.is_some() {
                return one(op, t[0].f_clamp_tensor(min_t.copied(), max_t.copied()));
            }
            match (p.scalar("min"), p.scalar("max")) {
                (None, None) => Err((
                    "bad_argument",
                    "clamp: at least one of --min/--max is required".to_string(),
                )),
                (Some(min), Some(max)) => one(op, t[0].f_clamp(to_scalar(min), to_scalar(max))),
                (Some(min), None) => one(op, t[0].f_clamp_min(to_scalar(min))),
                (None, Some(max)) => one(op, t[0].f_clamp_max(to_scalar(max))),
            }
        }
        "sum" => {
            crate::sum(t[0], p.int("dim"), p.bool("keepdim")).map(|t| Applied::Tensors(vec![t]))
        }
        "mean" => match p.int("dim") {
            // v1 fidelity: mean reduces in float32 regardless of input kind.
            Some(dim) => one(
                op,
                t[0].f_mean_dim(Some(&[dim][..]), p.bool("keepdim"), Kind::Float),
            ),
            None => one(op, t[0].f_mean(Kind::Float)),
        },
        "eq" => one(op, t[0].f_eq_tensor(t[1])),
        "allclose" => {
            let rtol = p.float("rtol").unwrap_or(1e-5);
            let atol = p.float("atol").unwrap_or(1e-8);
            Ok(Applied::Value(Data::Bool(
                t[0].f_allclose(t[1], rtol, atol, false)
                    .map_err(|e| tch(op, e))?,
            )))
        }
        "sort" => {
            let dim = p.int("dim").unwrap_or(-1);
            t[0].f_sort(dim, p.bool("descending"))
                .map(|(values, indices)| Applied::Tensors(vec![values, indices]))
                .map_err(|e| tch(op, e))
        }
        "mm" => {
            // Ported v1 validation: both rank-2, inner dims equal.
            let (sa, sb) = (t[0].size(), t[1].size());
            if sa.len() != 2 || sb.len() != 2 {
                return Err((
                    "shape_mismatch",
                    format!("mm: requires two 2-D tensors, got shapes {sa:?} and {sb:?}"),
                ));
            }
            if sa[1] != sb[0] {
                return Err((
                    "shape_mismatch",
                    format!("mm: inner dimensions must match, got {sa:?} and {sb:?}"),
                ));
            }
            one(op, t[0].f_mm(t[1]))
        }
        "cat" => {
            let dim = p.int("dim").unwrap_or(0);
            one(op, Tensor::f_cat(t, dim))
        }
        "full" => {
            let shape = p.int_list("shape").expect("required");
            validate_shape(op, &shape)?;
            let kind = parse_table_kind(op, p.str("dtype"))?;
            let result = match p.scalar("value").expect("required") {
                Scalar::Int(i) => Tensor::f_full(&shape, i, (kind, Device::Mps)),
                Scalar::Float(f) => Tensor::f_full(&shape, f, (kind, Device::Mps)),
            };
            let tensor = result.map_err(|e| tch(op, e))?;
            let tensor = if p.bool("requires_grad") {
                mark_requires_grad(tensor)?
            } else {
                tensor
            };
            Ok(Applied::Tensors(vec![tensor]))
        }
        "randn" => {
            let shape = p.int_list("shape").expect("required");
            validate_shape(op, &shape)?;
            let kind = parse_table_kind(op, p.str("dtype"))?;
            if !matches!(kind, Kind::Float | Kind::Half) {
                return Err((
                    "bad_dtype",
                    format!(
                        "randn: requires float32 or float16 (float64 is unsupported on MPS), got {kind:?}"
                    ),
                ));
            }
            // Generate on the seeded CPU generator, then transfer: tch's
            // manual_seed does NOT reach the MPS generator (discovered in
            // issue 0005 exp 1), and the CPU generator is the one Python's
            // torch.manual_seed drives too — so this is what makes randn
            // both deterministic and golden-comparable.
            let tensor = Tensor::f_randn(&shape, (kind, Device::Cpu))
                .and_then(|t| t.f_to_device(Device::Mps))
                .map_err(|e| tch(op, e))?;
            // requires_grad LAST, on the post-transfer tensor (the .to()
            // non-leaf trap, issue 0008): set before the move, the MPS
            // tensor is a non-leaf whose grad stays None forever.
            let tensor = if p.bool("requires_grad") {
                mark_requires_grad(tensor)?
            } else {
                tensor
            };
            Ok(Applied::Tensors(vec![tensor]))
        }
        // --- reductions sweep (issue 0005 exp 3) ---
        "prod" => match p.int("dim") {
            Some(dim) => one(
                op,
                t[0].f_prod_dim_int(dim, p.bool("keepdim"), None::<Kind>),
            ),
            None => one(op, t[0].f_prod(None::<Kind>)),
        },
        "amax" => match p.int("dim") {
            Some(dim) => one(op, t[0].f_amax(&[dim][..], p.bool("keepdim"))),
            None => one(op, t[0].f_amax(&[][..], p.bool("keepdim"))),
        },
        "amin" => match p.int("dim") {
            Some(dim) => one(op, t[0].f_amin(&[dim][..], p.bool("keepdim"))),
            None => one(op, t[0].f_amin(&[][..], p.bool("keepdim"))),
        },
        "max" | "min" | "median" => match p.int("dim") {
            Some(dim) => {
                let pair = match op {
                    "max" => t[0].f_max_dim(dim, p.bool("keepdim")),
                    "min" => t[0].f_min_dim(dim, p.bool("keepdim")),
                    _ => t[0].f_median_dim(dim, p.bool("keepdim")),
                };
                pair.map(|(values, indices)| Applied::Tensors(vec![values, indices]))
                    .map_err(|e| tch(op, e))
            }
            None => match op {
                "max" => one(op, t[0].f_max()),
                "min" => one(op, t[0].f_min()),
                _ => one(op, t[0].f_median()),
            },
        },
        "argmax" => one(op, t[0].f_argmax(p.int("dim"), p.bool("keepdim"))),
        "argmin" => one(op, t[0].f_argmin(p.int("dim"), p.bool("keepdim"))),
        "all" => match p.int("dim") {
            Some(dim) => one(op, t[0].f_all_dims(&[dim][..], p.bool("keepdim"))),
            None => one(op, t[0].f_all()),
        },
        "any" => match p.int("dim") {
            Some(dim) => one(op, t[0].f_any_dims(&[dim][..], p.bool("keepdim"))),
            None => one(op, t[0].f_any()),
        },
        "std" | "var" => {
            let correction: tch::Scalar = p.int("correction").unwrap_or(1).into();
            let dim_holder;
            let dim: Option<&[i64]> = match p.int("dim") {
                Some(d) => {
                    dim_holder = [d];
                    Some(&dim_holder[..])
                }
                None => None,
            };
            let result = if op == "std" {
                t[0].f_std_correction(dim, correction, p.bool("keepdim"))
            } else {
                t[0].f_var_correction(dim, correction, p.bool("keepdim"))
            };
            one(op, result)
        }
        "nansum" => {
            let dim_holder;
            let dim: Option<&[i64]> = match p.int("dim") {
                Some(d) => {
                    dim_holder = [d];
                    Some(&dim_holder[..])
                }
                None => None,
            };
            one(op, t[0].f_nansum(dim, p.bool("keepdim"), None::<Kind>))
        }
        "logsumexp" => one(
            op,
            t[0].f_logsumexp(&[p.int("dim").expect("required")][..], p.bool("keepdim")),
        ),
        "count_nonzero" => one(op, t[0].f_count_nonzero(p.int("dim"))),
        "cumsum" => one(
            op,
            t[0].f_cumsum(p.int("dim").expect("required"), None::<Kind>),
        ),
        "cumprod" => one(
            op,
            t[0].f_cumprod(p.int("dim").expect("required"), None::<Kind>),
        ),
        "norm" => {
            let pval = p.float("p").unwrap_or(2.0);
            match p.int("dim") {
                Some(dim) => one(
                    op,
                    t[0].f_norm_scalaropt_dim(pval, &[dim][..], p.bool("keepdim")),
                ),
                None => one(op, t[0].f_norm_scalaropt_dtype(pval, Kind::Float)),
            }
        }
        // --- comparison sweep ---
        "gt" => one(op, t[0].f_gt_tensor(t[1])),
        "lt" => one(op, t[0].f_lt_tensor(t[1])),
        "ge" => one(op, t[0].f_ge_tensor(t[1])),
        "le" => one(op, t[0].f_le_tensor(t[1])),
        "ne" => one(op, t[0].f_ne_tensor(t[1])),
        "logical_and" => one(op, t[0].f_logical_and(t[1])),
        "logical_or" => one(op, t[0].f_logical_or(t[1])),
        "logical_xor" => one(op, t[0].f_logical_xor(t[1])),
        "logical_not" => one(op, t[0].f_logical_not()),
        "isclose" => one(
            op,
            t[0].f_isclose(
                t[1],
                p.float("rtol").unwrap_or(1e-5),
                p.float("atol").unwrap_or(1e-8),
                false,
            ),
        ),
        "isnan" => one(op, t[0].f_isnan()),
        "isinf" => one(op, t[0].f_isinf()),
        "isfinite" => one(op, t[0].f_isfinite()),
        "isposinf" => one(op, t[0].f_isposinf()),
        "isneginf" => one(op, t[0].f_isneginf()),
        "equal" => t[0]
            .f_equal(t[1])
            .map(|b| Applied::Value(Data::Bool(b)))
            .map_err(|e| tch(op, e)),
        "topk" => t[0]
            .f_topk(
                p.int("k").expect("required"),
                p.int("dim").unwrap_or(-1),
                !p.bool("smallest"),
                true,
            )
            .map(|(values, indices)| Applied::Tensors(vec![values, indices]))
            .map_err(|e| tch(op, e)),
        "argsort" => one(
            op,
            t[0].f_argsort(p.int("dim").unwrap_or(-1), p.bool("descending")),
        ),
        // --- linalg + shape sweep (issue 0005 exp 4) ---
        "matmul" => one(op, t[0].f_matmul(t[1])),
        "bmm" => one(op, t[0].f_bmm(t[1])),
        "dot" => one(op, t[0].f_dot(t[1])),
        "outer" => one(op, t[0].f_outer(t[1])),
        "einsum" => one(
            op,
            Tensor::f_einsum(p.str("equation").expect("required"), t, None::<&[i64]>),
        ),
        "tril" => one(op, t[0].f_tril(p.int("diagonal").unwrap_or(0))),
        "triu" => one(op, t[0].f_triu(p.int("diagonal").unwrap_or(0))),
        "diag" => one(op, t[0].f_diag(p.int("diagonal").unwrap_or(0))),
        "trace" => one(op, t[0].f_trace()),
        "det" => one(op, t[0].f_det()),
        "inverse" => one(op, t[0].f_inverse()),
        "svd" => t[0]
            .f_svd(false, true)
            .map(|(u, s, v)| Applied::Tensors(vec![u, s, v]))
            .map_err(|e| tch(op, e)),
        "solve" => one(op, Tensor::f_linalg_solve(t[0], t[1], true)),
        "reshape" => one(op, t[0].f_reshape(p.int_list("shape").expect("required"))),
        "permute" => one(op, t[0].f_permute(p.int_list("dims").expect("required"))),
        "transpose" => one(
            op,
            t[0].f_transpose(
                p.int("dim0").expect("required"),
                p.int("dim1").expect("required"),
            ),
        ),
        "t" => {
            let size = t[0].size();
            if size.len() != 2 {
                return Err((
                    "shape_mismatch",
                    format!("t: requires a 2-D tensor, got shape {size:?}"),
                ));
            }
            one(op, t[0].f_tr())
        }
        "squeeze" => match p.int("dim") {
            Some(dim) => one(op, t[0].f_squeeze_dim(dim)),
            None => one(op, t[0].f_squeeze()),
        },
        "unsqueeze" => one(op, t[0].f_unsqueeze(p.int("dim").expect("required"))),
        "flatten" => one(
            op,
            t[0].f_flatten(
                p.int("start_dim").unwrap_or(0),
                p.int("end_dim").unwrap_or(-1),
            ),
        ),
        "stack" => one(op, Tensor::f_stack(t, p.int("dim").unwrap_or(0))),
        "split" => t[0]
            .f_split(
                p.int("split_size").expect("required"),
                p.int("dim").unwrap_or(0),
            )
            .map(Applied::Tensors)
            .map_err(|e| tch(op, e)),
        "chunk" => t[0]
            .f_chunk(
                p.int("chunks").expect("required"),
                p.int("dim").unwrap_or(0),
            )
            .map(Applied::Tensors)
            .map_err(|e| tch(op, e)),
        "gather" => one(
            op,
            t[0].f_gather(p.int("dim").expect("required"), t[1], false),
        ),
        "index_select" => one(
            op,
            t[0].f_index_select(p.int("dim").expect("required"), t[1]),
        ),
        "masked_select" => {
            // Numeric mask cast via != 0 (documented nutorch-ism: no bool
            // input path exists yet).
            let mask = t[1].f_ne(0).map_err(|e| tch(op, e))?;
            one(op, t[0].f_masked_select(&mask))
        }
        "where" => {
            // Numeric cond cast via != 0 (documented nutorch-ism).
            let cond = t[0].f_ne(0).map_err(|e| tch(op, e))?;
            one(op, t[1].f_where_self(&cond, t[2]))
        }
        "narrow" => one(
            op,
            t[0].f_narrow(
                p.int("dim").expect("required"),
                p.int("start").expect("required"),
                p.int("length").expect("required"),
            ),
        ),
        "flip" => one(op, t[0].f_flip(p.int_list("dims").expect("required"))),
        "roll" => {
            let shifts = p.int_list("shifts").expect("required");
            let dims = p.int_list("dims").unwrap_or_default();
            one(op, t[0].f_roll(&shifts, &dims))
        }
        "repeat" => one(op, t[0].f_repeat(p.int_list("repeats").expect("required"))),
        "repeat_interleave" => one(
            op,
            t[0].f_repeat_interleave_self_int(
                p.int("repeats").expect("required"),
                p.int("dim"),
                None,
            ),
        ),
        "movedim" => one(
            op,
            t[0].f_movedim(
                p.int("source").expect("required"),
                p.int("destination").expect("required"),
            ),
        ),
        // --- creation + remainder sweep (issue 0005 exp 5) ---
        "zeros" | "ones" => {
            let shape = p.int_list("shape").expect("required");
            validate_shape(op, &shape)?;
            let kind = parse_table_kind(op, p.str("dtype"))?;
            let result = if op == "zeros" {
                Tensor::f_zeros(&shape, (kind, Device::Mps))
            } else {
                Tensor::f_ones(&shape, (kind, Device::Mps))
            };
            let tensor = result.map_err(|e| tch(op, e))?;
            let tensor = if p.bool("requires_grad") {
                mark_requires_grad(tensor)?
            } else {
                tensor
            };
            Ok(Applied::Tensors(vec![tensor]))
        }
        "eye" => {
            let n = p.int("n").expect("required");
            match p.int("m") {
                Some(m) => one(op, Tensor::f_eye_m(n, m, (Kind::Float, Device::Mps))),
                None => one(op, Tensor::f_eye(n, (Kind::Float, Device::Mps))),
            }
        }
        "arange" => {
            let to_scalar = |s: Scalar| -> tch::Scalar {
                match s {
                    Scalar::Int(i) => i.into(),
                    Scalar::Float(f) => f.into(),
                }
            };
            let end = to_scalar(p.scalar("end").expect("required"));
            let start = to_scalar(p.scalar("start").unwrap_or(Scalar::Int(0)));
            let step = to_scalar(p.scalar("step").unwrap_or(Scalar::Int(1)));
            one(
                op,
                Tensor::f_arange_start_step(start, end, step, (Kind::Float, Device::Mps)),
            )
        }
        "linspace" => {
            let to_scalar = |s: Scalar| -> tch::Scalar {
                match s {
                    Scalar::Int(i) => i.into(),
                    Scalar::Float(f) => f.into(),
                }
            };
            one(
                op,
                Tensor::f_linspace(
                    to_scalar(p.scalar("start").expect("required")),
                    to_scalar(p.scalar("end").expect("required")),
                    p.int("steps").expect("required"),
                    (Kind::Float, Device::Mps),
                ),
            )
        }
        "rand" => {
            let shape = p.int_list("shape").expect("required");
            validate_shape(op, &shape)?;
            // Seeded CPU generator -> MPS (the randn convention);
            // requires_grad LAST, post-transfer (the .to() non-leaf trap).
            let tensor = Tensor::f_rand(&shape, (Kind::Float, Device::Cpu))
                .and_then(|t| t.f_to_device(Device::Mps))
                .map_err(|e| tch(op, e))?;
            let tensor = if p.bool("requires_grad") {
                mark_requires_grad(tensor)?
            } else {
                tensor
            };
            Ok(Applied::Tensors(vec![tensor]))
        }
        "randint" => {
            let shape = p.int_list("shape").expect("required");
            validate_shape(op, &shape)?;
            let low = p.int("low").unwrap_or(0);
            let high = p.int("high").expect("required");
            one(
                op,
                Tensor::f_randint_low(low, high, &shape, (Kind::Int64, Device::Cpu))
                    .and_then(|t| t.f_to_device(Device::Mps)),
            )
        }
        "zeros_like" => one(op, t[0].f_zeros_like()),
        "ones_like" => one(op, t[0].f_ones_like()),
        "full_like" => match p.scalar("value").expect("required") {
            Scalar::Int(i) => one(op, t[0].f_full_like(i)),
            Scalar::Float(f) => one(op, t[0].f_full_like(f)),
        },
        "rand_like" | "randn_like" => {
            // By-shape on the seeded CPU generator -> MPS (golden parity).
            let shape = t[0].size();
            let result = if op == "rand_like" {
                Tensor::f_rand(&shape, (Kind::Float, Device::Cpu))
            } else {
                Tensor::f_randn(&shape, (Kind::Float, Device::Cpu))
            };
            one(op, result.and_then(|x| x.f_to_device(Device::Mps)))
        }
        "lerp" => match pt.get("weight") {
            Some(weight) => one(op, t[0].f_lerp_tensor(t[1], weight)),
            None => match p.scalar("weight").expect("required") {
                Scalar::Int(i) => one(op, t[0].f_lerp(t[1], i)),
                Scalar::Float(f) => one(op, t[0].f_lerp(t[1], f)),
            },
        },
        "addcmul" | "addcdiv" => {
            // tch 0.24 exposes no `value` parameter for addcmul/addcdiv, so
            // the scaled form is computed manually: a + value * (b ∘ c).
            let combined = if op == "addcmul" {
                t[1].f_mul(t[2])
            } else {
                t[1].f_div(t[2])
            }
            .map_err(|e| tch(op, e))?;
            let result = match p.scalar("value") {
                None => t[0].f_add(&combined),
                Some(value) => {
                    let scaled = match value {
                        Scalar::Int(i) => combined.f_mul_scalar(i),
                        Scalar::Float(f) => combined.f_mul_scalar(f),
                    }
                    .map_err(|e| tch(op, e))?;
                    t[0].f_add(&scaled)
                }
            };
            one(op, result)
        }
        "cross" => one(op, t[0].f_cross(t[1], p.int("dim"))),
        "kron" => one(op, t[0].f_kron(t[1])),
        "tensordot" => {
            let dims = p.int("dims").unwrap_or(2);
            let axes: Vec<i64> = (0..dims).collect();
            let a_axes: Vec<i64> =
                (t[0].size().len() as i64 - dims..t[0].size().len() as i64).collect();
            one(op, t[0].f_tensordot(t[1], &a_axes, &axes))
        }
        "take_along_dim" => one(
            op,
            t[0].f_take_along_dim(t[1], p.int("dim").expect("required")),
        ),
        // ATen's searchsorted self is the VALUES tensor; our spec order is
        // (sorted_sequence, values), matching torch.searchsorted.
        "searchsorted" => one(
            op,
            t[1].f_searchsorted(t[0], false, false, "left", None::<&Tensor>),
        ),
        "bucketize" => one(op, t[0].f_bucketize(t[1], false, false)),
        "msort" => one(op, t[0].f_msort()),
        "diff" => one(
            op,
            t[0].f_diff(
                1,
                p.int("dim").unwrap_or(-1),
                None::<&Tensor>,
                None::<&Tensor>,
            ),
        ),
        "scatter" => one(
            op,
            t[0].f_scatter(p.int("dim").expect("required"), t[1], t[2]),
        ),
        "bitwise_and" => one(op, t[0].f_bitwise_and_tensor(t[1])),
        "bitwise_or" => one(op, t[0].f_bitwise_or_tensor(t[1])),
        "bitwise_xor" => one(op, t[0].f_bitwise_xor_tensor(t[1])),
        "bitwise_not" => one(op, t[0].f_bitwise_not()),
        "bitwise_left_shift" => one(op, t[0].f_bitwise_left_shift(t[1])),
        "bitwise_right_shift" => one(op, t[0].f_bitwise_right_shift(t[1])),
        // torch.unique flattens first; tch exposes no flattened f_unique,
        // so flatten explicitly then unique along dim 0 (a rank-2 golden
        // pins this — f_unique_dim(-1) alone diverges for rank >= 2).
        "unique" => t[0]
            .f_flatten(0, -1)
            .and_then(|flat| flat.f_unique_dim(0, true, false, false))
            .map(|(values, _, _)| Applied::Tensors(vec![values]))
            .map_err(|e| tch(op, e)),
        // --- autograd surface (issue 0008) ---
        "backward" => crate::backward(t[0]).map(|_| Applied::Nothing),
        "grad" => crate::grad(t[0]).map(|t| Applied::Tensors(vec![t])),
        "detach" => crate::detach(t[0]).map(|t| Applied::Tensors(vec![t])),
        "zero_grad" => crate::zero_grad(t[0]).map(|_| Applied::Nothing),
        // --- losses (issue 0009 exp 3) ---
        "mse_loss" => one(op, t[0].f_mse_loss(t[1], parse_reduction(p)?)),
        "l1_loss" => one(op, t[0].f_l1_loss(t[1], parse_reduction(p)?)),
        "smooth_l1_loss" => one(
            op,
            t[0].f_smooth_l1_loss(t[1], parse_reduction(p)?, p.float("beta").unwrap_or(1.0)),
        ),
        "huber_loss" => one(
            op,
            t[0].f_huber_loss(t[1], parse_reduction(p)?, p.float("delta").unwrap_or(1.0)),
        ),
        "cross_entropy" => one(
            op,
            t[0].f_cross_entropy_loss(t[1], None::<&Tensor>, parse_reduction(p)?, -100, 0.0),
        ),
        "nll_loss" => one(
            op,
            t[0].f_nll_loss(t[1], None::<&Tensor>, parse_reduction(p)?, -100),
        ),
        "binary_cross_entropy" => one(
            op,
            t[0].f_binary_cross_entropy(t[1], None::<&Tensor>, parse_reduction(p)?),
        ),
        "binary_cross_entropy_with_logits" => one(
            op,
            t[0].f_binary_cross_entropy_with_logits(
                t[1],
                None::<&Tensor>,
                None::<&Tensor>,
                parse_reduction(p)?,
            ),
        ),
        "kl_div" => one(
            op,
            t[0].f_kl_div(t[1], parse_reduction(p)?, p.bool("log_target")),
        ),
        "manual_seed" => {
            tch::manual_seed(p.int("seed").expect("required"));
            Ok(Applied::Nothing)
        }
        other => Err((
            "unknown_op",
            format!("table op {other} has no apply mapping (bug)"),
        )),
    }
}

fn validate_shape(op: &str, shape: &[i64]) -> Result<(), OpError> {
    if shape.is_empty() {
        return Err(("bad_argument", format!("{op}: shape cannot be empty")));
    }
    if let Some(bad) = shape.iter().find(|d| **d < 1) {
        return Err((
            "bad_argument",
            format!("{op}: every shape dimension must be >= 1, got {bad}"),
        ));
    }
    Ok(())
}

fn parse_table_kind(op: &str, dtype: Option<&str>) -> Result<Kind, OpError> {
    crate::parse_kind(dtype).map_err(|e| ("bad_dtype", format!("{op}: {e}")))
}
