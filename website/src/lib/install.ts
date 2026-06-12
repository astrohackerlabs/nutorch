// The canonical brew install block. The landing page imports it; the
// getting-started doc carries the same text in its first ```bash fence,
// and scripts/check-content.ts asserts they match byte for byte.
export const INSTALL = `brew tap nutorch/nutorch
brew trust nutorch/nutorch   # brew 6.0+ requires trusting third-party taps
brew install nutorch`;
