import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

/**
 * iQube Protocol Operations Manual Sidebar Structure
 * 
 * Designed for both business users and technical developers
 * Extensible structure for ongoing protocol development
 */
const sidebars: SidebarsConfig = {
  operationsManual: [
    'intro',
    {
      type: 'category',
      label: '🚀 Getting Started',
      items: [
        'getting-started/quick-start',
      ],
    },
    {
      type: 'category', 
      label: '📋 User Operations',
      items: [
        'user-operations/aigent-z-interface',
        'user-operations/iqube-operations',
        'user-operations/registry-management',
        'user-operations/network-ops',
      ],
    },
    {
      type: 'category',
      label: '🔧 System Operations',
      items: [
        'system-operations/monitoring',
      ],
    },
    {
      type: 'category',
      label: '🏗️ Technical Architecture',
      items: [
        'architecture/overview',
      ],
    },
    {
      type: 'category',
      label: '🔗 Integration Guide',
      items: [
        'integration/icp-canisters',
      ],
    },
    {
      type: 'category',
      label: '🛠️ Development',
      items: [
        'development/build-manual',
      ],
    },
    {
      type: 'category',
      label: '📚 Reference',
      items: [
        'reference/glossary',
      ],
    },
  ],
};

export default sidebars;
