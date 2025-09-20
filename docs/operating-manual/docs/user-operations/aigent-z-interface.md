---
sidebar_position: 1
title: Aigent Z Interface
description: Complete guide to using the Aigent Z unified intelligence agent platform
---

# Aigent Z Interface

Aigent Z is the unified intelligence agent platform that serves as the primary interface for interacting with the iQube Protocol. It provides a comprehensive suite of tools for managing iQubes, interacting with AI agents, and monitoring network operations.

## Interface Overview

### Sidebar Navigation

The Aigent Z interface features a persistent **Nakamoto-style sidebar** with manual section control:

- **Dashboard**: Main overview and analytics
- **Aigents**: AI persona interactions and chat
- **iQube Operations**: Core iQube management tools
- **Registry**: Browse and manage iQube templates and instances
- **Settings**: Configuration and Network Ops access

:::tip Navigation Control
Sidebar sections use manual expand/collapse control to prevent unwanted auto-expansion. Click section headers to toggle visibility of submenu items.
:::

### Global Features

#### Dark Theme
- Modern dark theme optimized for extended use
- Consistent styling across all components
- Enhanced readability and reduced eye strain

#### Keyboard Shortcuts
- Quick navigation between major sections
- Keyboard-accessible controls throughout the interface
- ARIA compliance for accessibility

#### Responsive Design
- Optimized for desktop, tablet, and mobile devices
- Adaptive layouts that maintain functionality across screen sizes
- Touch-friendly controls for mobile interaction

## Core Interface Sections

### Dashboard
The main dashboard provides:
- **System Overview**: High-level status of all integrated services
- **Recent Activity**: Latest iQube operations and transactions
- **Quick Actions**: Fast access to common operations
- **Analytics**: Usage metrics and performance indicators

### Aigents (AI Personas)
Specialized AI agent personas for different use cases:

#### Available Personas
- **metaMe**: Personal data and identity management
- **Additional Personas**: Context-specific agents for various domains

#### Features
- **Context Transformation Panel**: Modify agent behavior and focus
- **Persistent Chat History**: Maintain conversation context across sessions
- **Multi-Agent Collaboration**: Coordinate between different personas

### iQube Operations
Core functionality for managing iQubes through multiple operational modes:

#### Multi-Mode Tab Navigation
Color-coded tabs with ARIA accessibility:
- **View Mode**: Browse and inspect iQube metadata
- **Use Mode**: Populate instances from templates
- **Edit Mode**: Full template editing capabilities
- **Decrypt Mode**: Secure BlakQube data access
- **Mint Mode**: Convert templates to blockchain instances
- **Activate Mode**: Activate existing instances with codes

### Registry
Comprehensive iQube marketplace and management:
- **Browse Templates**: Discover available iQube templates
- **Instance Management**: Track and manage iQube instances
- **Library Operations**: Private local storage for drafts
- **Minting Workflows**: Convert templates to registry entries

### Settings
Configuration and system management:
- **Profile**: User preferences and account settings
- **Network Ops**: Real-time blockchain and canister monitoring

## User Experience Features

### Enhanced UI Consistency

#### Dynamic Price Display
- Real-time Sats pricing in MetaQube headers (Instance mode)
- Synchronized pricing between headers and data sections
- Green styling consistent with design system

#### iQube Name Display
- iQube names displayed in View tab headers
- Consistent naming across Template and Instance modes
- Clean headers without redundant "Template"/"Instance" text

#### Visual Indicators
- **Source Icons**: Visual indicators for different data source types
- **Status Badges**: Clear indication of iQube states and types
- **Score Indicators**: Visual representation of risk, accuracy, and other metrics

### State Management

#### Persistent State
- User preferences saved across sessions
- Sidebar section states maintained
- Form data preserved during navigation

#### Local Storage Integration
- Browser-based storage for immediate responsiveness
- Seamless integration with server-side persistence
- Conflict resolution between local and server state

## Stability and Performance

### Critical Bug Fixes

#### Sidebar Auto-Expansion Resolution
Previous versions experienced infinite update loops causing crashes. This has been resolved through:
- **Removed Auto-Expansion Logic**: Eliminated problematic automatic section opening
- **Manual Section Control**: Users now control section visibility explicitly
- **Separated Concerns**: Navigation-based opening separated from form interactions

#### Process Management
- Enhanced PM2 configuration with log rotation
- Auto-start functionality on system boot
- Improved development server stability

### Performance Optimizations
- **Efficient Re-rendering**: Optimized React component updates
- **Code Splitting**: Lazy loading for non-critical components
- **Caching Strategies**: Smart caching for frequently accessed data

## Accessibility Features

### ARIA Compliance
- Comprehensive ARIA labels and descriptions
- Screen reader compatibility
- Keyboard navigation support

### Visual Accessibility
- High contrast ratios in dark theme
- Clear visual hierarchy and typography
- Consistent iconography and visual cues

### Interaction Accessibility
- Keyboard shortcuts for all major functions
- Focus management for modal dialogs
- Clear error messages and validation feedback

## Integration Points

### Backend Services
- **Next.js API Routes**: Server-side processing and data management
- **Supabase Integration**: Database operations and user management
- **Blockchain Connectivity**: Direct integration with Web3 services

### External Services
- **ICP Canisters**: Real-time canister method calls
- **RPC Endpoints**: Live blockchain data from multiple networks
- **AI Services**: Integration with language models and reasoning engines

## Troubleshooting Common Issues

### Interface Loading Problems
1. **Clear Browser Cache**: Force refresh to load latest version
2. **Check Network Connectivity**: Verify connection to backend services
3. **Disable Browser Extensions**: Test with extensions disabled
4. **Update Browser**: Ensure modern browser with latest features

### Sidebar Issues
1. **Manual Refresh**: Hard refresh if sections don't respond
2. **Clear Local Storage**: Reset sidebar state if corrupted
3. **Check Console**: Review browser console for JavaScript errors

### Performance Issues
1. **Close Unused Tabs**: Reduce memory usage in browser
2. **Check System Resources**: Ensure adequate RAM and CPU
3. **Network Speed**: Verify adequate bandwidth for real-time updates

## Best Practices

### Efficient Workflow
- **Use Keyboard Shortcuts**: Faster navigation between sections
- **Organize iQubes**: Use Library for drafts, Registry for final versions
- **Regular Monitoring**: Check Network Ops for system health

### Data Management
- **Save Frequently**: Use Library to save work in progress
- **Backup Important Data**: Export critical iQubes regularly
- **Version Control**: Track changes to important templates

### Security Practices
- **Secure Sessions**: Log out when finished on shared computers
- **Private Data**: Use BlakQube encryption for sensitive information
- **Access Control**: Manage visibility settings carefully for public iQubes

## Future Enhancements

The Aigent Z interface continues to evolve with planned improvements:

- **Enhanced Multi-Agent Collaboration**: Better coordination between AI personas
- **Advanced Analytics**: Deeper insights into iQube usage and performance
- **Custom Dashboards**: User-configurable interface layouts
- **Mobile App**: Native mobile applications for iOS and Android
- **Voice Interface**: Voice commands for hands-free operation

---

*The Aigent Z interface provides a powerful, intuitive platform for all iQube Protocol operations while maintaining the flexibility to grow with user needs and protocol evolution.*
