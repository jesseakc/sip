import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // Docker needs a self-contained server (docker/Dockerfile.frontend copies
  // .next/standalone). Vercel builds its own output and does not use standalone,
  // so only ask for it when we are not building on Vercel — Vercel sets VERCEL=1.
  ...(process.env.VERCEL ? {} : { output: "standalone" as const }),
  images: {
    // No Next image optimiser in either target: Docker runs without sharp and the
    // frontend only renders operator-supplied URLs.
    unoptimized: true,
  },
};

export default nextConfig;
