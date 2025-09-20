---
sidebar_position: 1
title: Build Manual
description: Complete guide to building, deploying, and maintaining iQube Protocol applications
---

# Build Manual

This comprehensive build manual covers the complete development lifecycle for iQube Protocol applications, from initial setup through production deployment and maintenance.

## Development Environment Setup

### Prerequisites

#### System Requirements
- **Node.js**: Version 18.0 or higher
- **npm**: Version 9.0 or higher (or pnpm 8.0+)
- **Python**: Version 3.8 or higher
- **Rust**: Version 1.70 or higher
- **Git**: Latest version for version control

#### Platform-Specific Tools
- **macOS**: Xcode Command Line Tools
- **Linux**: build-essential package
- **Windows**: Visual Studio Build Tools

#### Blockchain Development Tools
- **dfx**: DFINITY Canister SDK for ICP development
- **Foundry**: Ethereum development framework (optional)
- **Web3 Wallet**: MetaMask or similar for testing

### Installation Steps

#### 1. Clone Repository
```bash
# Clone the monorepo
git clone https://github.com/iQube-Protocol/iQubeBeta-Program.git
cd iQubeBeta-Program

# Initialize submodules if any
git submodule update --init --recursive
```

#### 2. Install Dependencies
```bash
# Install Node.js dependencies (from monorepo root)
npm install

# Install Python dependencies
pip install -r requirements.txt

# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown
```

#### 3. Install dfx (ICP Development)
```bash
# Install DFINITY Canister SDK
sh -ci "$(curl -fsSL https://sdk.dfinity.org/install.sh)"

# Add to PATH
echo 'export PATH="$HOME/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# Verify installation
dfx --version
```

#### 4. Environment Configuration
```bash
# Copy environment template
cp .env.example .env.local

# Edit environment variables
nano .env.local
```

### Environment Variables

#### Required Configuration
```env
# Supabase Configuration (Required for Registry)
SUPABASE_URL=your_supabase_project_url
SUPABASE_ANON_KEY=your_supabase_anon_key
SUPABASE_SERVICE_ROLE_KEY=your_supabase_service_role_key
NEXT_PUBLIC_SUPABASE_URL=your_supabase_project_url
NEXT_PUBLIC_SUPABASE_ANON_KEY=your_supabase_anon_key

# Core API Configuration
NEXT_PUBLIC_CORE_API_URL=http://localhost:8000
CORE_API_KEY=your_api_key_here

# Registry API Configuration
NEXT_PUBLIC_REGISTRY_API_URL=http://localhost:8001
REGISTRY_API_KEY=your_registry_api_key_here

# Aigent Configuration
NEXT_PUBLIC_AIGENT_API_URL=http://localhost:8002
AIGENT_API_KEY=your_aigent_api_key_here

# Blockchain RPC Configuration
INFURA_PROJECT_ID=your_infura_project_id
POLYGON_RPC_URL=https://rpc-amoy.polygon.technology

# Feature Flags
NEXT_PUBLIC_FEATURE_OPS=true
NEXT_PUBLIC_FEATURE_TESTING=true
```

#### Development vs Production
```env
# Development
NODE_ENV=development
NEXT_PUBLIC_IC_HOST=http://localhost:4943
NEXT_PUBLIC_DFX_NETWORK=local

# Production
NODE_ENV=production
NEXT_PUBLIC_IC_HOST=https://ic0.app
NEXT_PUBLIC_DFX_NETWORK=ic
```

## Application Architecture

### Monorepo Structure
```
iQubeBeta-Program/                # Main monorepo root
├── apps/
│   ├── aigent-z/                 # Main Aigent Z application (formerly AigentZBeta)
│   ├── ops-console/              # Network Operations console
│   └── 21sats-market/            # 21 Sats marketplace
├── packages/
│   ├── sdk-js/                   # JavaScript SDK
│   └── indexers/                 # Blockchain indexers
├── canisters/                    # ICP canister source code
├── contracts/                    # Smart contracts
├── docs/
│   └── operating-manual/         # This documentation
└── vendor/                       # Third-party dependencies
```

:::info Monorepo Organization
The AigentZBeta application is now integrated as `apps/aigent-z/` within the iQubeBeta-Program monorepo structure. All development should be done within this monorepo context.
:::

### Aigent Z Application Structure
```
apps/aigent-z/                    # Aigent Z application directory
├── app/                          # Next.js 14 app directory
│   ├── aigents/                  # AI persona interfaces
│   ├── api/                      # API routes
│   │   ├── aigent/               # Aigent API endpoints
│   │   ├── core/                 # Core API endpoints
│   │   ├── ops/                  # Operations monitoring
│   │   └── registry/             # Registry management
│   ├── dashboard/                # Main dashboard
│   ├── iqube/                    # iQube operations
│   ├── ops/                      # Network operations
│   ├── registry/                 # Registry interface
│   ├── settings/                 # User settings
│   └── test/                     # Testing dashboard
├── components/                   # Shared UI components
│   ├── registry/                 # Registry-specific components
│   ├── ui/                       # Base UI components
│   └── ops/                      # Operations components
├── hooks/                        # Custom React hooks
├── lib/                          # Utility libraries
├── public/                       # Static assets
├── styles/                       # Global styles
├── docs/                         # Application-specific documentation
│   ├── PROGRESS_REPORT.md        # Development progress tracking
│   └── OPERATORS_MANUAL.md       # Legacy operators manual
├── BUILD_MANUAL.md               # Application build instructions
├── README.md                     # Application overview
└── ARCHITECTURE.md               # Application architecture details
```

## Build Process

### Development Build

#### Start Development Servers
```bash
# Start all applications in development mode
npm run dev

# Or start specific applications
npm run dev:aigent-z
npm run dev:ops-console
npm run dev:21sats

# Start with specific port
PORT=3001 npm run dev:aigent-z
```

#### Start ICP Local Replica
```bash
# Start local Internet Computer replica
dfx start --background

# Deploy canisters locally
dfx deploy

# Check canister status
dfx canister status --all
```

#### Unified Development Script
```bash
# Make script executable
chmod +x start_dev.sh

# Start both Python backend and Next.js frontend
./start_dev.sh
```

### Production Build

#### Build Applications
```bash
# Build all applications
npm run build

# Build specific application
npm run build:aigent-z

# Build with environment
NODE_ENV=production npm run build
```

#### Build Optimization
```bash
# Analyze bundle size
npm run analyze

# Build with source maps (for debugging)
npm run build:debug

# Build without source maps (for production)
npm run build:production
```

#### Static Export (if needed)
```bash
# Export static files
npm run export

# Serve static files
npm run serve
```

### Canister Deployment

#### Local Deployment
```bash
# Deploy to local replica
dfx deploy

# Deploy specific canister
dfx deploy proof_of_state

# Deploy with arguments
dfx deploy --argument '(record { admin = principal "your-principal" })'
```

#### Production Deployment
```bash
# Deploy to IC mainnet
dfx deploy --network ic

# Deploy with cycles
dfx deploy --with-cycles 1000000000000

# Check deployment status
dfx canister status --network ic --all
```

## Testing

### Unit Testing

#### Frontend Tests
```bash
# Run all tests
npm test

# Run tests in watch mode
npm run test:watch

# Run tests with coverage
npm run test:coverage

# Run specific test file
npm test -- --testPathPattern=components
```

#### Canister Tests
```bash
# Run Rust canister tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_issue_receipt
```

### Integration Testing

#### End-to-End Tests
```bash
# Run E2E tests with Playwright
npm run test:e2e

# Run E2E tests in headed mode
npm run test:e2e:headed

# Run specific E2E test
npm run test:e2e -- --grep "minting workflow"
```

#### API Testing
```bash
# Test API endpoints
npm run test:api

# Test with different environments
NODE_ENV=test npm run test:api
```

### System Testing

#### Health Checks
```bash
# Run system health tests
npm run test:health

# Test blockchain connectivity
npm run test:blockchain

# Test canister connectivity
npm run test:canisters
```

#### Load Testing
```bash
# Run load tests
npm run test:load

# Test with specific parameters
npm run test:load -- --users 100 --duration 60s
```

## Code Quality

### Linting and Formatting

#### ESLint Configuration
```bash
# Run ESLint
npm run lint

# Fix ESLint issues
npm run lint:fix

# Lint specific files
npm run lint -- --ext .ts,.tsx src/
```

#### Prettier Configuration
```bash
# Format code
npm run format

# Check formatting
npm run format:check

# Format specific files
npm run format -- "src/**/*.{ts,tsx}"
```

#### TypeScript Checking
```bash
# Type check
npm run type-check

# Type check in watch mode
npm run type-check:watch
```

### Pre-commit Hooks

#### Husky Configuration
```json
{
  "husky": {
    "hooks": {
      "pre-commit": "lint-staged",
      "pre-push": "npm run type-check && npm test"
    }
  }
}
```

#### Lint-staged Configuration
```json
{
  "lint-staged": {
    "*.{ts,tsx}": [
      "eslint --fix",
      "prettier --write",
      "git add"
    ],
    "*.{js,jsx}": [
      "eslint --fix",
      "prettier --write",
      "git add"
    ]
  }
}
```

## Deployment

### Vercel Deployment (Frontend)

#### Automatic Deployment
```bash
# Connect to Vercel (first time)
npx vercel

# Deploy to preview
npx vercel

# Deploy to production
npx vercel --prod
```

#### Environment Variables
Configure in Vercel dashboard:
- All `NEXT_PUBLIC_*` variables
- API keys and secrets
- Database connection strings

#### Build Configuration
```json
{
  "buildCommand": "npm run build",
  "outputDirectory": ".next",
  "installCommand": "npm install",
  "framework": "nextjs"
}
```

### Docker Deployment

#### Dockerfile
```dockerfile
FROM node:18-alpine AS base

# Install dependencies
FROM base AS deps
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production

# Build application
FROM base AS builder
WORKDIR /app
COPY . .
COPY --from=deps /app/node_modules ./node_modules
RUN npm run build

# Production image
FROM base AS runner
WORKDIR /app
ENV NODE_ENV production

RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 nextjs

COPY --from=builder /app/public ./public
COPY --from=builder --chown=nextjs:nodejs /app/.next/standalone ./
COPY --from=builder --chown=nextjs:nodejs /app/.next/static ./.next/static

USER nextjs
EXPOSE 3000
ENV PORT 3000

CMD ["node", "server.js"]
```

#### Docker Compose
```yaml
version: '3.8'
services:
  aigent-z:
    build: ./apps/aigent-z
    ports:
      - "3000:3000"
    environment:
      - NODE_ENV=production
    depends_on:
      - postgres
      
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: iqube
      POSTGRES_USER: iqube
      POSTGRES_PASSWORD: password
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

### ICP Canister Deployment

#### Production Deployment Script
```bash
#!/bin/bash
# deploy-canisters.sh

set -e

echo "Deploying canisters to IC mainnet..."

# Build canisters
dfx build --network ic

# Deploy with cycles
dfx deploy --network ic --with-cycles 10000000000000

# Verify deployment
dfx canister status --network ic --all

echo "Deployment complete!"
```

#### Canister Upgrade
```bash
# Upgrade canister with new code
dfx canister install --mode upgrade --network ic proof_of_state

# Upgrade with arguments
dfx canister install --mode upgrade --network ic --argument '(record { version = "2.0.0" })' proof_of_state
```

## Monitoring and Maintenance

### Application Monitoring

#### Health Checks
```typescript
// /app/api/health/route.ts
export async function GET() {
  const checks = {
    database: await checkDatabase(),
    canisters: await checkCanisters(),
    external_apis: await checkExternalAPIs(),
  };
  
  const healthy = Object.values(checks).every(check => check.status === 'ok');
  
  return Response.json({
    status: healthy ? 'healthy' : 'unhealthy',
    checks,
    timestamp: new Date().toISOString(),
  }, {
    status: healthy ? 200 : 503,
  });
}
```

#### Performance Monitoring
```typescript
// Performance monitoring with Web Vitals
import { getCLS, getFID, getFCP, getLCP, getTTFB } from 'web-vitals';

function sendToAnalytics(metric) {
  // Send to your analytics service
  console.log(metric);
}

getCLS(sendToAnalytics);
getFID(sendToAnalytics);
getFCP(sendToAnalytics);
getLCP(sendToAnalytics);
getTTFB(sendToAnalytics);
```

### Canister Monitoring

#### Cycles Management
```bash
# Check canister cycles
dfx canister status --network ic proof_of_state

# Top up canister cycles
dfx canister deposit-cycles 1000000000000 proof_of_state --network ic

# Monitor cycles usage
dfx canister call proof_of_state get_cycles_balance
```

#### Performance Metrics
```rust
// Canister performance monitoring
#[query]
fn get_performance_metrics() -> PerformanceMetrics {
    PerformanceMetrics {
        cycles_balance: ic_cdk::api::canister_balance(),
        memory_usage: ic_cdk::api::stable::stable_size(),
        heap_memory: ic_cdk::api::performance_counter(1),
        instruction_count: ic_cdk::api::performance_counter(0),
        call_count: CALL_COUNT.with(|c| c.get()),
    }
}
```

### Backup and Recovery

#### Database Backups
```bash
# Automated backup script
#!/bin/bash
DATE=$(date +%Y%m%d_%H%M%S)
pg_dump $DATABASE_URL > "backup_${DATE}.sql"
aws s3 cp "backup_${DATE}.sql" s3://iqube-backups/
```

#### Code Backups
```bash
# Create full backup
./scripts/create_backup.sh

# Restore from backup
./scripts/restore_from_backup.sh --backup 20250918_143000
```

#### Canister State Backup
```bash
# Export canister state
dfx canister call proof_of_state export_state > state_backup.json

# Import canister state
dfx canister call proof_of_state import_state "$(cat state_backup.json)"
```

## Troubleshooting

### Common Build Issues

#### Node.js Version Conflicts
```bash
# Use Node Version Manager
nvm install 18
nvm use 18

# Verify version
node --version
npm --version
```

#### Dependency Conflicts
```bash
# Clear npm cache
npm cache clean --force

# Remove node_modules and reinstall
rm -rf node_modules package-lock.json
npm install

# Use npm ci for clean install
npm ci
```

#### TypeScript Errors
```bash
# Check TypeScript configuration
npx tsc --noEmit

# Update TypeScript
npm update typescript @types/node @types/react

# Clear TypeScript cache
rm -rf .next/cache
```

### Runtime Issues

#### Environment Variable Problems
```bash
# Check environment variables
node -e "console.log(process.env)"

# Verify Next.js environment
npm run build && npm start
```

#### API Connection Issues
```bash
# Test API connectivity
curl -v http://localhost:3000/api/health

# Check network connectivity
ping ic0.app
nslookup ic0.app
```

#### Canister Connection Issues
```bash
# Check canister status
dfx canister status --network ic --all

# Test canister calls
dfx canister call proof_of_state get_pending_count --network ic

# Check IC network status
curl -v https://ic-api.internetcomputer.org/api/v3/metrics
```

### Performance Issues

#### Bundle Size Optimization
```bash
# Analyze bundle
npm run analyze

# Check for duplicate dependencies
npx duplicate-package-checker

# Optimize images
npx next-optimized-images
```

#### Memory Leaks
```bash
# Profile memory usage
node --inspect app.js

# Check for memory leaks in tests
npm test -- --detectLeaks
```

## Best Practices

### Development Workflow
1. **Feature Branches**: Use feature branches for all development
2. **Code Reviews**: Require code reviews for all changes
3. **Testing**: Write tests for all new features
4. **Documentation**: Update documentation with changes

### Security Practices
1. **Environment Variables**: Never commit secrets to version control
2. **Dependencies**: Regularly update and audit dependencies
3. **Input Validation**: Validate all inputs on both client and server
4. **Error Handling**: Implement proper error handling and logging

### Performance Optimization
1. **Code Splitting**: Use dynamic imports for large components
2. **Caching**: Implement appropriate caching strategies
3. **Bundle Optimization**: Minimize bundle size and optimize assets
4. **Database Optimization**: Use proper indexing and query optimization

---

*This build manual provides comprehensive guidance for developing, building, and maintaining iQube Protocol applications across the entire development lifecycle.*
