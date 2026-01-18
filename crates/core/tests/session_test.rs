use sisyphus_core::session::manager::SessionManager;

#[tokio::test]
async fn test_session_creation() {
    let manager = SessionManager::new();

    assert!(manager.list_sessions().is_empty());
}

#[tokio::test]
async fn test_session_creation_with_agent_id() {
    let manager = SessionManager::new();

    let agent_id = Some("test-agent-id".to_string());
    let session = manager.create_session(agent_id.clone());

    let session_data = session.read().await;
    assert_eq!(session_data.agent_id, agent_id);
}

#[tokio::test]
async fn test_session_status_transitions() {
    let manager = SessionManager::new();

    let session = manager.create_session(None);

    let session_ref = session.read().await;

    assert_eq!(
        session_ref.status,
        sisyphus_core::session::SessionStatus::Idle
    );
}

#[tokio::test]
async fn test_session_manager_list_sessions() {
    let manager = SessionManager::new();

    manager.create_session(Some("agent-1".to_string()));
    manager.create_session(Some("agent-2".to_string()));

    let sessions = manager.list_sessions();

    assert_eq!(sessions.len(), 2);
}

#[tokio::test]
async fn test_session_manager_get_session() {
    let manager = SessionManager::new();

    let session = manager.create_session(Some("agent-1".to_string()));
    let session_id = session.read().await.id.clone();

    let retrieved = manager.get_session(&session_id);

    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().read().await.id, session_id);
}

#[tokio::test]
async fn test_session_manager_get_nonexistent() {
    let manager = SessionManager::new();

    let retrieved = manager.get_session("nonexistent-id");

    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_multiple_session_managers_isolation() {
    let manager1 = SessionManager::new();
    let manager2 = SessionManager::new();

    let session1 = manager1.create_session(Some("agent-1".to_string()));
    manager2.create_session(Some("agent-2".to_string()));

    assert_eq!(manager1.list_sessions().len(), 1);
    assert_eq!(manager2.list_sessions().len(), 1);

    let session_id = session1.read().await.id.clone();
    let retrieved = manager1.get_session(&session_id);
    assert!(retrieved.is_some());

    let session2 = manager2.get_session(&session_id);
    assert!(session2.is_none());
}

#[tokio::test]
async fn test_session_id_uniqueness() {
    let manager = SessionManager::new();

    let session1 = manager.create_session(None);
    let session2 = manager.create_session(None);
    let session3 = manager.create_session(None);

    let ids = vec![
        session1.read().await.id.clone(),
        session2.read().await.id.clone(),
        session3.read().await.id.clone(),
    ];

    let unique_ids: std::collections::HashSet<_> = ids.into_iter().collect();

    assert_eq!(unique_ids.len(), 3);
}

#[tokio::test]
async fn test_session_concurrent_access() {
    let manager = SessionManager::new();

    let session = manager.create_session(Some("agent-1".to_string()));
    let session_id = session.read().await.id.clone();

    tokio::spawn(async move {
        let mut s = session.write().await;
        s.status = sisyphus_core::session::SessionStatus::Busy;
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let retrieved = manager.get_session(&session_id);
    assert!(retrieved.is_some());

    let retrieved_session = retrieved.unwrap();
    let session_data = retrieved_session.read().await;

    assert_eq!(
        session_data.status,
        sisyphus_core::session::SessionStatus::Busy
    );
}
