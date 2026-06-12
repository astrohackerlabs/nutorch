---
title: Pointwise ops
description: The 71 pointwise operations, generated from the op table.
order: 21
section: "Reference"
---

Generated from the binaries by `scripts/gen-ops-reference.ts` — do not edit by
hand. Every op also documents itself: `torch <op> --help`.

### add

a + alpha*b (broadcasting; --alpha default 1)

```bash
torch add <t1> <t2> [--alpha <Scalar>]
```

```nu
nutorch add <t1> <t2> [--alpha <Scalar>]
```

### sub

a - alpha*b (broadcasting; --alpha default 1)

```bash
torch sub <t1> <t2> [--alpha <Scalar>]
```

```nu
nutorch sub <t1> <t2> [--alpha <Scalar>]
```

### sin

elementwise sine

```bash
torch sin <t1>
```

```nu
nutorch sin <t1>
```

### pow

elementwise power (scalar or tensor exponent)

```bash
torch pow <t1> <exponent>
```

```nu
nutorch pow <t1> <exponent>
```

### clamp

clamp into [min, max] (scalar or tensor bounds; one required)

```bash
torch clamp <t1> [--min <HandleOrScalar>] [--max <HandleOrScalar>]
```

```nu
nutorch clamp <t1> [--min <HandleOrScalar>] [--max <HandleOrScalar>]
```

### abs

elementwise absolute value

```bash
torch abs <t1>
```

```nu
nutorch abs <t1>
```

### acos

elementwise arccosine

```bash
torch acos <t1>
```

```nu
nutorch acos <t1>
```

### acosh

elementwise inverse hyperbolic cosine

```bash
torch acosh <t1>
```

```nu
nutorch acosh <t1>
```

### asin

elementwise arcsine

```bash
torch asin <t1>
```

```nu
nutorch asin <t1>
```

### asinh

elementwise inverse hyperbolic sine

```bash
torch asinh <t1>
```

```nu
nutorch asinh <t1>
```

### atan

elementwise arctangent

```bash
torch atan <t1>
```

```nu
nutorch atan <t1>
```

### atanh

elementwise inverse hyperbolic tangent

```bash
torch atanh <t1>
```

```nu
nutorch atanh <t1>
```

### ceil

elementwise ceiling

```bash
torch ceil <t1>
```

```nu
nutorch ceil <t1>
```

### cos

elementwise cosine

```bash
torch cos <t1>
```

```nu
nutorch cos <t1>
```

### cosh

elementwise hyperbolic cosine

```bash
torch cosh <t1>
```

```nu
nutorch cosh <t1>
```

### deg2rad

degrees to radians

```bash
torch deg2rad <t1>
```

```nu
nutorch deg2rad <t1>
```

### digamma

elementwise digamma

```bash
torch digamma <t1>
```

```nu
nutorch digamma <t1>
```

### erf

elementwise error function

```bash
torch erf <t1>
```

```nu
nutorch erf <t1>
```

### erfc

elementwise complementary error function

```bash
torch erfc <t1>
```

```nu
nutorch erfc <t1>
```

### exp

elementwise e^x

```bash
torch exp <t1>
```

```nu
nutorch exp <t1>
```

### exp2

elementwise 2^x

```bash
torch exp2 <t1>
```

```nu
nutorch exp2 <t1>
```

### expm1

elementwise e^x - 1

```bash
torch expm1 <t1>
```

```nu
nutorch expm1 <t1>
```

### floor

elementwise floor

```bash
torch floor <t1>
```

```nu
nutorch floor <t1>
```

### frac

elementwise fractional part

```bash
torch frac <t1>
```

```nu
nutorch frac <t1>
```

### i0

elementwise modified Bessel function I0

```bash
torch i0 <t1>
```

```nu
nutorch i0 <t1>
```

### lgamma

elementwise log-gamma

```bash
torch lgamma <t1>
```

```nu
nutorch lgamma <t1>
```

### log

elementwise natural log

```bash
torch log <t1>
```

```nu
nutorch log <t1>
```

### log10

elementwise log base 10

```bash
torch log10 <t1>
```

```nu
nutorch log10 <t1>
```

### log1p

elementwise log(1+x)

```bash
torch log1p <t1>
```

```nu
nutorch log1p <t1>
```

### log2

elementwise log base 2

```bash
torch log2 <t1>
```

```nu
nutorch log2 <t1>
```

### logit

elementwise logit (inverse sigmoid)

```bash
torch logit <t1>
```

```nu
nutorch logit <t1>
```

### neg

elementwise negation

```bash
torch neg <t1>
```

```nu
nutorch neg <t1>
```

### rad2deg

radians to degrees

```bash
torch rad2deg <t1>
```

```nu
nutorch rad2deg <t1>
```

### reciprocal

elementwise 1/x

```bash
torch reciprocal <t1>
```

```nu
nutorch reciprocal <t1>
```

### relu

elementwise max(x, 0)

```bash
torch relu <t1>
```

```nu
nutorch relu <t1>
```

### round

elementwise round to nearest

```bash
torch round <t1>
```

```nu
nutorch round <t1>
```

### rsqrt

elementwise 1/sqrt(x)

```bash
torch rsqrt <t1>
```

```nu
nutorch rsqrt <t1>
```

### sgn

elementwise sign (complex-aware)

```bash
torch sgn <t1>
```

```nu
nutorch sgn <t1>
```

### sigmoid

elementwise sigmoid

```bash
torch sigmoid <t1>
```

```nu
nutorch sigmoid <t1>
```

### sign

elementwise sign

```bash
torch sign <t1>
```

```nu
nutorch sign <t1>
```

### sinc

elementwise normalized sinc

```bash
torch sinc <t1>
```

```nu
nutorch sinc <t1>
```

### sinh

elementwise hyperbolic sine

```bash
torch sinh <t1>
```

```nu
nutorch sinh <t1>
```

### sqrt

elementwise square root

```bash
torch sqrt <t1>
```

```nu
nutorch sqrt <t1>
```

### square

elementwise x^2

```bash
torch square <t1>
```

```nu
nutorch square <t1>
```

### tan

elementwise tangent

```bash
torch tan <t1>
```

```nu
nutorch tan <t1>
```

### tanh

elementwise hyperbolic tangent

```bash
torch tanh <t1>
```

```nu
nutorch tanh <t1>
```

### trunc

elementwise truncation toward zero

```bash
torch trunc <t1>
```

```nu
nutorch trunc <t1>
```

### softmax

softmax along --dim (float32)

```bash
torch softmax <t1> [--dim <Int>]
```

```nu
nutorch softmax <t1> [--dim <Int>]
```

### log_softmax

log-softmax along --dim (float32)

```bash
torch log_softmax <t1> [--dim <Int>]
```

```nu
nutorch log_softmax <t1> [--dim <Int>]
```

### nan_to_num

replace NaN/inf (--nan/--posinf/--neginf)

```bash
torch nan_to_num <t1> [--nan <Float>] [--posinf <Float>] [--neginf <Float>]
```

```nu
nutorch nan_to_num <t1> [--nan <Float>] [--posinf <Float>] [--neginf <Float>]
```

### mul

elementwise product (broadcasting)

```bash
torch mul <t1> <t2>
```

```nu
nutorch mul <t1> <t2>
```

### div

elementwise true division (broadcasting)

```bash
torch div <t1> <t2>
```

```nu
nutorch div <t1> <t2>
```

### maximum

elementwise maximum (broadcasting)

```bash
torch maximum <t1> <t2>
```

```nu
nutorch maximum <t1> <t2>
```

### minimum

elementwise minimum (broadcasting)

```bash
torch minimum <t1> <t2>
```

```nu
nutorch minimum <t1> <t2>
```

### atan2

elementwise atan2(a, b) (broadcasting)

```bash
torch atan2 <t1> <t2>
```

```nu
nutorch atan2 <t1> <t2>
```

### fmod

elementwise C-style remainder (broadcasting)

```bash
torch fmod <t1> <t2>
```

```nu
nutorch fmod <t1> <t2>
```

### remainder

elementwise Python-style remainder (broadcasting)

```bash
torch remainder <t1> <t2>
```

```nu
nutorch remainder <t1> <t2>
```

### floor_divide

elementwise floor division (broadcasting)

```bash
torch floor_divide <t1> <t2>
```

```nu
nutorch floor_divide <t1> <t2>
```

### hypot

elementwise hypotenuse (broadcasting)

```bash
torch hypot <t1> <t2>
```

```nu
nutorch hypot <t1> <t2>
```

### copysign

magnitude of a, sign of b (broadcasting)

```bash
torch copysign <t1> <t2>
```

```nu
nutorch copysign <t1> <t2>
```

### xlogy

elementwise x*log(y) (broadcasting)

```bash
torch xlogy <t1> <t2>
```

```nu
nutorch xlogy <t1> <t2>
```

### logaddexp

elementwise log(e^a + e^b) (broadcasting)

```bash
torch logaddexp <t1> <t2>
```

```nu
nutorch logaddexp <t1> <t2>
```

### lerp

a + weight*(b - a) (scalar or tensor weight)

```bash
torch lerp <t1> <t2> <weight>
```

```nu
nutorch lerp <t1> <t2> <weight>
```

### addcmul

a + value * b * c

```bash
torch addcmul <t1> <t2> <t3> [--value <Scalar>]
```

```nu
nutorch addcmul <t1> <t2> <t3> [--value <Scalar>]
```

### addcdiv

a + value * b / c

```bash
torch addcdiv <t1> <t2> <t3> [--value <Scalar>]
```

```nu
nutorch addcdiv <t1> <t2> <t3> [--value <Scalar>]
```

### bitwise_and

bitwise AND of int tensors (broadcasting)

```bash
torch bitwise_and <t1> <t2>
```

```nu
nutorch bitwise_and <t1> <t2>
```

### bitwise_or

bitwise OR of int tensors (broadcasting)

```bash
torch bitwise_or <t1> <t2>
```

```nu
nutorch bitwise_or <t1> <t2>
```

### bitwise_xor

bitwise XOR of int tensors (broadcasting)

```bash
torch bitwise_xor <t1> <t2>
```

```nu
nutorch bitwise_xor <t1> <t2>
```

### bitwise_not

bitwise NOT of an int tensor

```bash
torch bitwise_not <t1>
```

```nu
nutorch bitwise_not <t1>
```

### bitwise_left_shift

left shift of int tensors (broadcasting)

```bash
torch bitwise_left_shift <t1> <t2>
```

```nu
nutorch bitwise_left_shift <t1> <t2>
```

### bitwise_right_shift

right shift of int tensors (broadcasting)

```bash
torch bitwise_right_shift <t1> <t2>
```

```nu
nutorch bitwise_right_shift <t1> <t2>
```
