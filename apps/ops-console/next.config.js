/** @type {import('next').NextConfig} */
const nextConfig = {
  experimental: {
    appDir: true,
  },
  transpilePackages: ['@iqube/sdk-js'],
  webpack: (config) => {
    // Enable WebAssembly for tiny-secp256k1
    config.experiments = config.experiments || {};
    config.experiments.asyncWebAssembly = true;
    return config;
  },
}

module.exports = nextConfig
