// Test script to verify live SDK integration
import { getAnchorStatus, getDualLockStatus, submitForAnchoring } from './packages/sdk-js/dist/index.js';

async function testLiveIntegration() {
  console.log('Testing live SDK integration...');
  
  try {
    // Test getting anchor status
    console.log('\n1. Testing getAnchorStatus...');
    const anchorStatus = await getAnchorStatus('test-receipt-id');
    console.log('Anchor Status:', anchorStatus);
    
    // Test getting dual lock status  
    console.log('\n2. Testing getDualLockStatus...');
    const dualLockStatus = await getDualLockStatus('test-message-id');
    console.log('Dual Lock Status:', dualLockStatus);
    
    // Test submitting for anchoring
    console.log('\n3. Testing submitForAnchoring...');
    const submitResult = await submitForAnchoring('test_data_live_sdk', 'test metadata');
    console.log('Submit Result:', submitResult);
    
    console.log('\n✅ Live SDK integration test completed!');
  } catch (error) {
    console.error('❌ Test failed:', error);
  }
}

testLiveIntegration();
