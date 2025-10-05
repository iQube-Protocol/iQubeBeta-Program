# Staging Environment Setup Guide

## 🚨 **URGENT: Supabase Configuration Missing in Staging**

The staging site at `staging-beta.aigent-z.me` is showing "Supabase env not configured" because the deployment platform doesn't have the required environment variables.

## 📋 **Required Environment Variables for Staging**

### **Supabase Configuration (CRITICAL)**
```bash
# Primary Supabase Configuration
SUPABASE_URL=https://bsjhfvctmduxhohtllly.supabase.co
SUPABASE_ANON_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImJzamhmdmN0bWR1eGhvaHRsbGx5Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3NTc1NDgyNTgsImV4cCI6MjA3MzEyNDI1OH0.JVDp4-F6EEXqVQ8sts2Z8KQg168aZ1YdtY53RRM_s7M
SUPABASE_SERVICE_ROLE_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImJzamhmdmN0bWR1eGhvaHRsbGx5Iiwicm9sZSI6InNlcnZpY2Vfcm9sZSIsImlhdCI6MTc1NzU0ODI1OCwiZXhwIjoyMDczMTI0MjU4fQ.Ex0TywZI7QD7i3KcGkwK_xsSU9SZqwDBT7nlpaQ59ng

# Public Supabase Configuration (for client-side)
NEXT_PUBLIC_SUPABASE_URL=https://bsjhfvctmduxhohtllly.supabase.co
NEXT_PUBLIC_SUPABASE_ANON_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImJzamhmdmN0bWR1eGhvaHRsbGx5Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3NTc1NDgyNTgsImV4cCI6MjA3MzEyNDI1OH0.JVDp4-F6EEXqVQ8sts2Z8KQg168aZ1YdtY53RRM_s7M
```

### **ICP Canister Configuration (LIVE MAINNET)**
```bash
# Cross Chain Service (DVN) - LIVE MAINNET
CROSS_CHAIN_SERVICE_CANISTER_ID=sp5ye-2qaaa-aaaao-qkqla-cai
NEXT_PUBLIC_CROSS_CHAIN_SERVICE_CANISTER_ID=sp5ye-2qaaa-aaaao-qkqla-cai

# Proof of State - LIVE MAINNET
PROOF_OF_STATE_CANISTER_ID=n2hhv-aaaaa-aaaas-qccza-cai
NEXT_PUBLIC_PROOF_OF_STATE_CANISTER_ID=n2hhv-aaaaa-aaaas-qccza-cai

# Bitcoin Signer - LIVE MAINNET
BTC_SIGNER_CANISTER_ID=uxrrr-q7777-77774-qaaaq-cai
NEXT_PUBLIC_BTC_SIGNER_CANISTER_ID=uxrrr-q7777-77774-qaaaq-cai

# EVM RPC - LIVE MAINNET
EVM_RPC_CANISTER_ID=uzt4z-lp777-77774-qaabq-cai
NEXT_PUBLIC_EVM_RPC_CANISTER_ID=uzt4z-lp777-77774-qaabq-cai
```

### **ICP Network Configuration**
```bash
# ICP Network Configuration
DFX_NETWORK=ic
ICP_HOST=https://ic0.app
NEXT_PUBLIC_ICP_HOST=https://ic0.app
```

### **Application Configuration**
```bash
# Environment
NODE_ENV=production
NEXT_PUBLIC_NODE_ENV=production
NEXT_PUBLIC_ENVIRONMENT=staging

# Features
NEXT_PUBLIC_FEATURE_QCT_CROSS_TRADE_CARD=true

# Debug (optional for staging)
DEBUG=false
NEXT_PUBLIC_DEBUG=false
```

## 🔧 **Platform-Specific Setup Instructions**

### **Vercel Deployment**
1. Go to Vercel Dashboard → Project Settings → Environment Variables
2. Add all variables above with appropriate scopes:
   - `SUPABASE_*` variables: **Production, Preview, Development**
   - `NEXT_PUBLIC_*` variables: **Production, Preview, Development**
3. Redeploy the application

### **Netlify Deployment**
1. Go to Netlify Dashboard → Site Settings → Environment Variables
2. Add all variables above
3. Trigger a new deployment

### **Custom Server/VPS**
1. Create `.env.production` file with all variables
2. Ensure the deployment process loads these variables
3. Restart the application

## 🔍 **Debugging Environment Issues**

### **Check Current Environment Status**
Visit: `https://staging-beta.aigent-z.me/api/registry/templates`

The response will now include debugging information:
```json
{
  "error": "Supabase env not configured",
  "debug": {
    "SUPABASE_URL": false,
    "NEXT_PUBLIC_SUPABASE_URL": false,
    "SUPABASE_ANON_KEY": false,
    "NEXT_PUBLIC_SUPABASE_ANON_KEY": false,
    "SUPABASE_SERVICE_ROLE_KEY": false,
    "NODE_ENV": "production",
    "VERCEL": true,
    "NETLIFY": false
  },
  "message": "Please configure SUPABASE_URL and SUPABASE_ANON_KEY environment variables in your deployment platform"
}
```

### **Fallback Behavior**
The updated code now includes fallback values for Supabase configuration, so the Registry should work even without explicit environment variables. However, for security and reliability, it's recommended to set them explicitly.

## ✅ **Verification Steps**

1. **Add Environment Variables** to your deployment platform
2. **Redeploy** the application
3. **Test Registry**: Visit `/registry` page - should load without errors
4. **Check API**: Visit `/api/registry/templates` - should return data
5. **Test Creation**: Try creating a new template in the Registry

## 🚨 **Security Notes**

- **Never commit** `.env.local` or `.env.production` to version control
- **Service Role Key** should only be used server-side (not in `NEXT_PUBLIC_*` variables)
- **Anon Key** is safe for client-side use
- **Rotate keys** regularly for security

## 📞 **Support**

If the Registry still doesn't work after adding environment variables:

1. Check browser console for errors
2. Check deployment platform logs
3. Verify all environment variables are set correctly
4. Ensure the deployment was successful after adding variables

The updated code includes comprehensive fallback logic and debugging information to help identify and resolve configuration issues.
