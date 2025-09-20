---
sidebar_position: 2
title: Quick Start Guide
description: Get up and running with the iQube Protocol in minutes
---

# Quick Start Guide

Get started with the iQube Protocol in just a few minutes. This guide will walk you through accessing the applications, understanding the basic concepts, and performing your first operations.

## Prerequisites

### System Requirements
- **Modern Web Browser**: Chrome, Firefox, Safari, or Edge (latest versions)
- **Internet Connection**: Stable connection for real-time blockchain data
- **Web3 Wallet** (optional): MetaMask or similar for advanced features

### Network Access
The iQube Protocol applications are accessible at:
- **Aigent Z**: `http://localhost:3000` (development)
- **Network Ops**: Integrated within Aigent Z at Settings → Network Ops
- **Testing Dashboard**: `http://localhost:3000/test`

## Step 1: Access Aigent Z

### Launch the Application
1. Open your web browser
2. Navigate to the Aigent Z application URL
3. Wait for the application to load (dark theme interface)
4. The sidebar navigation will appear on the left

### Interface Overview
The main interface includes:
- **📊 Dashboard**: System overview and analytics
- **🤖 Aigents**: AI persona interactions
- **📦 iQube Operations**: Core iQube management
- **📚 Registry**: Template and instance marketplace
- **⚙️ Settings**: Configuration and Network Ops

## Step 2: Explore iQube Operations

### Understanding iQubes
iQubes are decentralized information assets with three components:
- **MetaQube**: Public metadata (always visible)
- **BlakQube**: Private encrypted data (access controlled)
- **TokenQube**: Access control and permissions

### Access iQube Operations
1. Click **iQube Operations** in the sidebar
2. The interface will show multiple operational modes:
   - **View**: Browse and inspect iQubes
   - **Use**: Create instances from templates
   - **Edit**: Full template customization
   - **Decrypt**: Access encrypted data
   - **Mint**: Convert to blockchain instances
   - **Activate**: Enable iQube functionality

### Try View Mode
1. Click the **View** tab
2. Browse available iQube templates
3. Click on any template to see its structure
4. Notice the MetaQube (public) and BlakQube (encrypted) sections
5. Observe the scoring system (Sensitivity, Accuracy, Verifiability, Risk)

## Step 3: Explore the Registry

### Access the Registry
1. Click **Registry** in the sidebar
2. The Registry displays available iQube templates and instances
3. Use the view toggle to switch between Grid, List, and Table views

### Browse Templates
1. **Filter by Type**: Use the filter controls to find specific iQube types
2. **View Details**: Click on any template card to see detailed information
3. **Understand Badges**: Notice the different badges indicating:
   - iQube Type (DataQube, ContentQube, etc.)
   - Business Model (Buy, Sell, Rent, etc.)
   - Visibility Status (Library, Registry Public/Private)

### Save to Library
1. Find an interesting template
2. Click the **Save to Library** button
3. The template is now saved locally in your browser
4. Notice the badge changes to "Library (Private)"

## Step 4: Monitor System Health

### Access Network Ops
1. Go to **Settings** in the sidebar
2. Click **Network Ops**
3. The Network Ops dashboard will load with real-time status cards

### Understand Status Cards
Each card shows:
- **🟢 Green Badge**: Service is active and healthy
- **🔴 Red Badge**: Service has issues or is unreachable
- **Response Time**: API call latency in milliseconds
- **Last Updated**: When data was last refreshed

### Monitor Live Services
The dashboard shows real-time status for:
- **Ethereum Sepolia**: Live testnet monitoring
- **Polygon Amoy**: Live testnet monitoring
- **ICP DVN**: Cross-chain service canister
- **BTC Anchor**: Bitcoin testnet integration
- **ICP Canister Health**: Overall canister status

### Use System Diagnostics
1. Click **Manual Refresh** to update all status cards immediately
2. Click **Testing Dashboard** for comprehensive system validation
3. Monitor response times to identify performance issues

## Step 5: Try the Testing Dashboard

### Access Testing Interface
1. From Network Ops, click **Testing Dashboard**
2. Or navigate directly to `/test` in your browser
3. The comprehensive testing interface will load

### Run System Health Tests
1. Click **Run All Tests** to validate all integrated services
2. Watch real-time test results appear
3. Green checkmarks indicate successful tests
4. Red X marks indicate issues that need attention

### Individual Service Tests
1. **Ethereum Sepolia Test**: Validates RPC connectivity
2. **Polygon Amoy Test**: Confirms testnet accessibility
3. **ICP DVN Test**: Tests canister connectivity
4. **BTC Anchor Test**: Verifies Bitcoin integration

### Application Feature Tests
1. **Minting Workflow Test**: End-to-end iQube minting validation
2. **Custom iQube Testing**: Test specific iQube IDs
3. **Token ID Testing**: Validate token interactions

## Step 6: Interact with AI Personas

### Access Aigents
1. Click **Aigents** in the sidebar
2. Browse available AI personas
3. Each persona has specialized capabilities and knowledge

### Start a Conversation
1. Select a persona (e.g., metaMe)
2. Type a message in the chat interface
3. The AI will respond with contextual information
4. Use the Context Transformation panel to modify AI behavior

### Explore Persona Capabilities
- **metaMe**: Personal data and identity management
- **Additional Personas**: Specialized for different domains
- **Context Awareness**: Personas understand iQube context
- **Persistent History**: Conversations are saved across sessions

## Step 7: Create Your First iQube

### Start with Use Mode
1. Go to **iQube Operations** → **Use** tab
2. Select a template from the available options
3. The interface will show template structure and fields

### Populate Template Fields
1. **MetaQube Section**: Fill in public metadata
   - iQube Name
   - Creator information
   - Description
   - Scoring values (1-10 scale)
2. **BlakQube Section**: Add private data fields
   - Personal information
   - Sensitive data
   - Custom field values

### Save Your Work
1. **Save to Library**: Store locally for further editing
2. **Validate**: Ensure all required fields are complete
3. **Preview**: Review your iQube before finalizing

## Step 8: Understanding Visibility and Minting

### Visibility Options
- **Library (Private)**: Stored locally in your browser only
- **Registry (Private)**: Stored on server, visible only to you
- **Registry (Public)**: Publicly visible and discoverable

### Minting Process
1. Complete your iQube in Use or Edit mode
2. Click **Mint to Registry**
3. Choose visibility (Public or Private)
4. Confirm the irreversible action
5. Wait for blockchain transaction completion

:::warning Important
Public minting is irreversible. Once an iQube is minted as public, it cannot be made private again.
:::

## Next Steps

### Explore Advanced Features
- **Edit Mode**: Create custom templates from scratch
- **Decrypt Mode**: Access encrypted BlakQube data
- **Activate Mode**: Use activation codes for special iQubes

### Learn More
- **[iQube Operations Guide](../user-operations/iqube-operations)**: Detailed operational procedures
- **[Network Ops Guide](../user-operations/network-ops)**: System monitoring and diagnostics
- **[Architecture Overview](../architecture/overview)**: Technical system details

### Get Help
- **Testing Dashboard**: Validate system health when issues occur
- **Network Ops**: Monitor real-time system status
- **Browser Console**: Check for detailed error messages
- **Documentation**: Comprehensive guides for all features

## Troubleshooting Quick Issues

### Application Won't Load
1. **Check Network Connection**: Ensure stable internet connectivity
2. **Clear Browser Cache**: Force refresh with Ctrl+F5 (Windows) or Cmd+Shift+R (Mac)
3. **Try Different Browser**: Test with another browser
4. **Check Console**: Look for JavaScript errors in browser developer tools

### Status Cards Show Red
1. **Wait for Refresh**: Status updates every 30 seconds automatically
2. **Manual Refresh**: Click the Manual Refresh button
3. **Check Testing Dashboard**: Run individual service tests
4. **Network Issues**: Verify internet connectivity to external services

### iQube Operations Not Working
1. **Refresh Page**: Reload the application
2. **Clear Local Storage**: Reset browser storage if needed
3. **Check Network Ops**: Verify system health
4. **Try Different Mode**: Switch between View, Use, Edit modes

---

*You're now ready to explore the full capabilities of the iQube Protocol! This quick start covers the basics - dive deeper into specific features using the detailed guides in this manual.*
