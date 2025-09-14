/** @type {import('next').NextConfig} */
const nextConfig = {
  experimental: {
    appDir: true,
  },
  transpilePackages: ['@iqube/sdk-js'],
}

module.exports = nextConfig
