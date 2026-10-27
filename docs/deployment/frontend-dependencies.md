# Frontend dependency upgrade

The frontend now uses Next.js 16.4.3 and patched React packages, pinned in package.json and the lockfile. The Next.js 15.5.28 candidate still had an affected transitive PostCSS dependency, so this update moves to the patched 16.x line. Node.js 22 is used by the container and CI.

Run `npm ci`, `npm audit --omit=dev`, `npm run typecheck`, and `npm run build` from frontend. Both the audit and production build passed locally. Verify the Vercel production and preview environments as part of deployment; no deployment was performed by this PR.

The Next.js build updated tsconfig's JSX runtime and generated-types includes. The obsolete `next lint` script is replaced with the accurately named `typecheck` script. A dedicated ESLint policy remains separate work.
