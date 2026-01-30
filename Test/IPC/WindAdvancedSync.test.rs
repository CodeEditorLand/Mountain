//! # Wind Advanced Sync Tests
//! 
//! Tests for the Wind Advanced Synchronization engine

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_sync_engine_initialization() {
        // Test that the sync engine initializes correctly
        let runtime = Arc::new(ApplicationRunTime::new());
        let sync = WindAdvancedSync::new(runtime);
        
        assert!(sync.initialize().await.is_ok());
    }

    #[tokio::test]
    async fn test_document_synchronization() {
        // Test document synchronization functionality
        let runtime = Arc::new(ApplicationRunTime::new());
        let sync = WindAdvancedSync::new(runtime);
        
        // Add a document for synchronization
        assert!(sync.add_document("test-doc-1".to_string(), "/test/path/file.txt".to_string()).await.is_ok());
        
        // Get sync status
        let status = sync.get_sync_status().await;
        assert_eq!(status.total_documents, 1);
        assert_eq!(status.synced_documents, 1);
    }

    #[tokio::test]
    async fn test_conflict_detection() {
        // Test conflict detection functionality
        let runtime = Arc::new(ApplicationRunTime::new());
        let sync = WindAdvancedSync::new(runtime);
        
        // Add a document
        sync.add_document("test-doc-2".to_string(), "/test/path/file2.txt".to_string()).await.unwrap();
        
        // Create a change that should trigger conflict detection
        let change = DocumentChange {
            change_id: "test-change-1".to_string(),
            document_id: "test-doc-2".to_string(),
            change_type: ChangeType::Update,
            content: Some("test content".to_string()),
            applied: false,
        };
        
        // Test conflict detection
        let result = sync.check_for_conflicts(&change).await;
        assert!(result.is_ok()); // Should not detect conflict for new document
    }

    #[tokio::test]
    async fn test_performance_monitoring() {
        // Test performance monitoring functionality
        let runtime = Arc::new(ApplicationRunTime::new());
        let sync = WindAdvancedSync::new(runtime);
        
        // Get initial performance stats
        let stats = sync.performance_stats.lock().unwrap();
        assert_eq!(stats.total_messages_sent, 0);
        assert_eq!(stats.total_messages_received, 0);
        drop(stats);
        
        // Simulate some activity
        sync.add_document("test-doc-3".to_string(), "/test/path/file3.txt".to_string()).await.unwrap();
        
        // Check that stats were updated
        let stats = sync.performance_stats.lock().unwrap();
        assert!(stats.last_update > 0);
    }

    #[tokio::test]
    async fn test_real_time_updates() {
        // Test real-time update functionality
        let runtime = Arc::new(ApplicationRunTime::new());
        let sync = WindAdvancedSync::new(runtime);
        
        // Subscribe to updates
        assert!(sync.subscribe_to_updates("test-target".to_string(), "test-subscriber".to_string()).await.is_ok());
        
        // Queue an update
        let update = RealTimeUpdate {
            target: "test-target".to_string(),
            data: "test data".to_string(),
        };
        assert!(sync.queue_update(update).await.is_ok());
    }

    #[tokio::test]
    async fn test_error_recovery() {
        // Test error recovery functionality
        let runtime = Arc::new(ApplicationRunTime::new());
        let sync = WindAdvancedSync::new(runtime);
        
        // Test with invalid document ID (should trigger error recovery)
        let change = DocumentChange {
            change_id: "test-change-2".to_string(),
            document_id: "non-existent-doc".to_string(),
            change_type: ChangeType::Update,
            content: Some("test content".to_string()),
            applied: false,
        };
        
        // This should fail but trigger error recovery
        let result = sync.apply_document_change(change).await;
        assert!(result.is_err()); // Should fail for non-existent document
    }

    #[tokio::test]
    async fn test_background_sync_task() {
        // Test background synchronization task
        let runtime = Arc::new(ApplicationRunTime::new());
        let sync = WindAdvancedSync::new(runtime);
        
        // Initialize sync
        sync.initialize().await.unwrap();
        
        // Add some documents
        sync.add_document("bg-doc-1".to_string(), "/bg/path/file1.txt".to_string()).await.unwrap();
        sync.add_document("bg-doc-2".to_string(), "/bg/path/file2.txt".to_string()).await.unwrap();
        
        // Wait for background sync to run
        sleep(Duration::from_millis(100)).await;
        
        // Check that sync status is updated
        let status = sync.get_sync_status().await;
        assert_eq!(status.total_documents, 2);
    }

    #[tokio::test]
    async fn test_ui_state_synchronization() {
        // Test UI state synchronization
        let runtime = Arc::new(ApplicationRunTime::new());
        let sync = WindAdvancedSync::new(runtime);
        
        // Get initial UI state
        let initial_state = sync.get_current_ui_state().await;
        assert_eq!(initial_state.theme, "default");
        
        // Update UI state
        let mut new_state = initial_state.clone();
        new_state.theme = "dark".to_string();
        
        assert!(sync.update_ui_state(new_state).await.is_ok());
        
        // Verify UI state was updated
        let updated_state = sync.get_current_ui_state().await;
        assert_eq!(updated_state.theme, "dark");
    }

    #[tokio::test]
    async fn test_sync_status_calculation() {
        // Test sync status calculation
        let runtime = Arc::new(ApplicationRunTime::new());
        let sync = WindAdvancedSync::new(runtime);
        
        // Add documents with different sync states
        sync.add_document("status-doc-1".to_string(), "/status/path/file1.txt".to_string()).await.unwrap();
        sync.add_document("status-doc-2".to_string(), "/status/path/file2.txt".to_string()).await.unwrap();
        
        // Manually set one document to conflicted state
        {
            let mut doc_sync = sync.document_sync.lock().unwrap();
            if let Some(doc) = doc_sync.synchronized_documents.get_mut("status-doc-2") {
                doc.sync_state = SyncState::Conflicted;
            }
        }
        
        // Update sync status
        sync.update_sync_status().await;
        
        // Check calculated status
        let status = sync.get_sync_status().await;
        assert_eq!(status.total_documents, 2);
        assert_eq!(status.synced_documents, 1);
        assert_eq!(status.conflicted_documents, 1);
    }
}