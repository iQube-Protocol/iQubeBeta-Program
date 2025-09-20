---
sidebar_position: 3
title: Registry Management
description: Complete guide to managing iQubes through the Registry interface
---

# Registry Management

The Registry serves as the central marketplace and management system for iQube templates and instances. It provides comprehensive tools for discovering, creating, managing, and sharing iQubes across the protocol ecosystem.

## Registry Overview

### Purpose and Function
The Registry acts as:
- **Marketplace**: Discover and access iQube templates created by others
- **Library**: Manage your personal collection of iQubes
- **Publishing Platform**: Share your iQube templates with the community
- **Version Control**: Track template evolution and instance relationships

### Visibility Levels
The Registry supports multiple visibility levels:
- **Library (Private)**: Local browser storage, visible only to you
- **Registry (Private)**: Server-side storage, visible only to owner
- **Registry (Public)**: Publicly visible and discoverable by all users

## Interface Overview

### View Modes
The Registry interface supports three distinct view modes:

#### Grid View
- **Layout**: Card-based layout with visual thumbnails
- **Information**: Key metadata, scores, and badges
- **Best For**: Browsing and visual discovery
- **Features**: Hover effects, quick actions, visual score indicators

#### List View
- **Layout**: Compact list format with essential information
- **Information**: Name, type, creator, and key metrics
- **Best For**: Scanning large numbers of iQubes
- **Features**: Sortable columns, quick filtering

#### Table View
- **Layout**: Comprehensive tabular display
- **Information**: All metadata fields and detailed scores
- **Best For**: Detailed analysis and comparison
- **Features**: All score columns, business model details, action buttons

### Navigation Controls
- **View Toggle**: Switch between Grid, List, and Table views
- **Filter Controls**: Filter by type, business model, and other criteria
- **Search**: Text-based search across iQube names and descriptions
- **Sort Options**: Sort by name, date, scores, or other attributes

## Discovering iQubes

### Browsing Templates
1. **Access Registry**: Click "Registry" in the main sidebar
2. **Choose View Mode**: Select Grid, List, or Table view based on preference
3. **Apply Filters**: Use filter controls to narrow down results
4. **Browse Results**: Scroll through available templates

### Search Functionality
- **Text Search**: Enter keywords in the search bar
- **Filter Combination**: Combine search with filters for precise results
- **Real-time Results**: Search results update as you type
- **Search Scope**: Searches names, descriptions, and creator information

### Understanding iQube Cards

#### Badge System
Each iQube displays multiple badges indicating:
- **Type Badge**: DataQube, ContentQube, AgentQube, etc. (colored by type)
- **Business Model Badge**: Buy, Sell, Rent, Subscribe, etc.
- **Visibility Badge**: Library (Private), Registry (Public/Private)
- **Status Indicators**: Special status like "Popular" or "Featured"

#### Score Visualization
Scores are displayed using a 5-dot system with color coding:
- **Sensitivity**: Red dots (higher = more sensitive)
- **Risk**: Yellow/orange dots (higher = more risky)
- **Accuracy**: Green dots (higher = more accurate)
- **Verifiability**: Green dots (higher = more verifiable)
- **Trust**: Green dots (calculated composite score)
- **Reliability**: Green dots (calculated composite score)

#### Action Buttons
Each iQube card provides quick action buttons:
- **👁️ View**: Open in View mode for inspection
- **✏️ Edit**: Open in Edit mode (if you have permissions)
- **🛒 Add to Cart**: Add to selection for batch operations
- **🗑️ Delete**: Remove from registry (if you own it)

## Managing Your iQubes

### Library Operations

#### Saving to Library
1. **Find Interesting Template**: Browse or search for templates
2. **Save Action**: Click "Save to Library" button on template card
3. **Local Storage**: Template is saved to browser localStorage
4. **Badge Update**: Card badge changes to "Library (Private)"
5. **Offline Access**: Template remains available even when offline

#### Library Benefits
- **Private Storage**: Only visible to you on your device
- **Draft Workspace**: Perfect for work-in-progress iQubes
- **No Server Dependency**: Works without internet connection
- **Quick Access**: Immediate availability for editing and use

#### Managing Library Items
- **View Library**: Filter by "Library" to see only your saved items
- **Edit Locally**: Modify templates without affecting original
- **Remove from Library**: Delete local copies when no longer needed
- **Mint to Registry**: Convert library items to registry entries

### Registry Operations

#### Minting to Registry
The minting process converts templates into permanent registry entries:

1. **Complete Template**: Ensure all required fields are filled
2. **Initiate Minting**: Click "Mint to Registry" button
3. **Choose Visibility**: Select Public or Private registry
4. **Confirmation Dialog**: Review and confirm the irreversible action
5. **Blockchain Transaction**: Wait for transaction completion
6. **Registry Update**: iQube appears in registry with new status

#### Visibility Selection
When minting, you must choose visibility:

**Public Registry**
- ✅ Visible to all users
- ✅ Discoverable through search and browsing
- ✅ Can be forked and used by others
- ❌ Cannot be made private later
- ❌ Irreversible decision

**Private Registry**
- ✅ Visible only to you
- ✅ Stored securely on server
- ✅ Can be activated to public later
- ✅ Maintains privacy until activated
- ❌ Not discoverable by others

:::warning Important
Public minting is irreversible. Once an iQube is minted as public, it cannot be made private again. Choose carefully!
:::

### Template Management

#### Creating Custom Templates
1. **Access Creation**: Click "Create Custom iQube Template"
2. **Template Form**: Fill out comprehensive template information
3. **Basic Information**: Name, creator, description
4. **Scoring Assessment**: Set sensitivity, accuracy, verifiability, risk scores
5. **Pricing & Rights**: Configure business model and pricing
6. **Schema Definition**: Define BlakQube data structure
7. **Save or Mint**: Save to library or mint directly to registry

#### Template Sections
**Basic Information**
- **iQube Name**: Descriptive name for the template
- **Creator**: Your name or organization
- **Description**: Detailed description of purpose and content
- **Owner Type**: Individual or Organization
- **iQube Type**: DataQube, ContentQube, AgentQube, etc.

**Scoring & Risk Assessment**
- **Sensitivity Score** (1-10): How sensitive is the data?
- **Verifiability Score** (1-10): How easily can data be verified?
- **Accuracy Score** (1-10): How accurate is the information?
- **Risk Score** (1-10): What risks are associated with this data?

**Pricing & Rights**
- **Business Model**: Buy, Sell, Rent, Subscribe, etc.
- **Price**: Monetary value for access
- **Duration of Rights**: Forever, Per Use, Time-based, etc.
- **Public Wallet Key**: Payment address for transactions

**Wallet & Schema**
- **BlakQube Schema**: Define private data structure
- **Transaction Date**: When template was created
- **Additional Metadata**: Custom fields and properties

#### Template Validation
Before minting, templates undergo validation:
- **Required Fields**: All mandatory fields must be completed
- **Score Ranges**: All scores must be between 1-10
- **Data Consistency**: Related fields must be compatible
- **Schema Validity**: BlakQube schema must be well-formed

### Instance Management

#### Understanding Template-Instance Relationships
- **Templates**: Reusable blueprints with defined structure
- **Instances**: Specific implementations with populated data
- **Inheritance**: Instances maintain connection to source template
- **Independence**: Instances can be modified without affecting template

#### Creating Instances
1. **Select Template**: Choose template from registry or library
2. **Use Mode**: Open template in Use mode
3. **Configure Instance**: Set instance-specific parameters
4. **Populate Data**: Fill in MetaQube and BlakQube fields
5. **Instance Numbering**: System assigns instance numbers (e.g., "3 of 21")
6. **Save or Mint**: Save as draft or mint to registry

#### Instance Features
- **Numbered Instances**: Automatic counting (e.g., "5 of 100")
- **Template Lineage**: Clear connection to source template
- **Independent Editing**: Modify without affecting template
- **Batch Creation**: Generate multiple instances efficiently

## Advanced Registry Features

### Analytics and Insights

#### Usage Metrics
The Registry provides analytics on:
- **View Counts**: How many times iQubes have been viewed
- **Usage Statistics**: How often templates are used to create instances
- **Popular Templates**: Most frequently accessed templates
- **Creator Statistics**: Performance metrics for template creators

#### Performance Tracking
- **Score Trends**: How iQube scores change over time
- **User Engagement**: Interaction patterns and preferences
- **Market Analysis**: Business model effectiveness
- **Quality Metrics**: Template and instance quality indicators

### Collaboration Features

#### Forking and Derivation
- **Fork Templates**: Create your own version of existing templates
- **Maintain Attribution**: Original creator information preserved
- **Independent Evolution**: Modify forked templates freely
- **Version Tracking**: Track relationship between original and fork

#### Sharing and Distribution
- **Public Sharing**: Make templates available to community
- **Private Sharing**: Share with specific users or groups
- **Licensing**: Define usage terms and restrictions
- **Attribution**: Ensure proper credit for creators

### Quality Assurance

#### Community Moderation
- **Reporting System**: Report inappropriate or low-quality content
- **Quality Reviews**: Community-driven quality assessment
- **Verification Badges**: Indicate verified or high-quality templates
- **Curation**: Featured and recommended template collections

#### Automated Validation
- **Schema Validation**: Ensure data structures are well-formed
- **Content Scanning**: Automated checks for inappropriate content
- **Quality Metrics**: Automated quality scoring
- **Duplicate Detection**: Identify and handle duplicate templates

## Best Practices

### Template Creation
1. **Clear Naming**: Use descriptive, searchable names
2. **Comprehensive Descriptions**: Provide detailed usage information
3. **Accurate Scoring**: Honestly assess sensitivity, risk, and accuracy
4. **Complete Schemas**: Define all necessary BlakQube fields
5. **Test Thoroughly**: Validate templates before public minting

### Registry Management
1. **Regular Maintenance**: Keep your templates updated and relevant
2. **Community Engagement**: Respond to user feedback and questions
3. **Quality Focus**: Prioritize quality over quantity
4. **Documentation**: Provide clear usage instructions
5. **Version Control**: Track changes and maintain version history

### Privacy and Security
1. **Visibility Decisions**: Carefully consider public vs private implications
2. **Sensitive Data**: Use BlakQube encryption for private information
3. **Access Control**: Implement appropriate capability tokens
4. **Regular Audits**: Review and update access permissions
5. **Backup Strategy**: Maintain backups of important templates

## Troubleshooting

### Common Issues

#### Templates Not Appearing
- **Check Filters**: Ensure filters aren't hiding your templates
- **Refresh Browser**: Clear cache and refresh the page
- **Verify Minting**: Confirm minting transaction completed successfully
- **Check Visibility**: Ensure you're looking in the right visibility category

#### Minting Failures
- **Complete Validation**: Ensure all required fields are filled
- **Check Network**: Verify blockchain connectivity
- **Sufficient Funds**: Ensure adequate gas/fees for transaction
- **Retry Transaction**: Attempt minting again after resolving issues

#### Search Not Working
- **Clear Search**: Remove search terms and try again
- **Check Spelling**: Verify search terms are spelled correctly
- **Use Filters**: Combine search with filters for better results
- **Browser Issues**: Try different browser or clear cache

#### Visibility Problems
- **Understand Levels**: Review visibility level definitions
- **Check Permissions**: Verify you have access to view certain iQubes
- **Login Status**: Ensure you're properly authenticated
- **Network Issues**: Check connection to registry services

---

*The Registry provides a comprehensive platform for discovering, creating, and managing iQubes, enabling a vibrant ecosystem of shared decentralized information assets.*
