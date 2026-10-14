/** @type {import('next').NextConfig} */
const nextConfig = {
  // Verification builds set NEXT_DIST_DIR so they never clobber a running `next dev`.
  distDir: process.env.NEXT_DIST_DIR || ".next",
  webpack: (config, { webpack }) => {
    // @solana/web3.js and Anchor expect Node's Buffer in the browser.
    config.resolve.fallback = { ...config.resolve.fallback, fs: false, path: false, os: false, crypto: false };
    config.plugins.push(new webpack.ProvidePlugin({ Buffer: ["buffer", "Buffer"] }));
    return config;
  },
};
export default nextConfig;
