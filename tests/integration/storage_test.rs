use clipflow::storage::repository::{add_clip, get_all_clips, delete_clip, clear_all_clips};
use clipflow::storage::rotation::rotate_if_needed;
use clipflow::pipeline::ClipItem;
use chrono::Utc;

#[tokio::test]
async fn test_add_and_get_clips() {
    let item = ClipItem {
        id: "test-1".to_string(),
        r#type: "text".to_string(),
        content: "Test content".to_string(),
        preview: "Test content".to_string(),
        timestamp: Utc::now().timestamp_millis(),
        size: 12,
        ocr_text: None,
        metadata: None,
    };

    add_clip(item.clone()).await.unwrap();
    let clips = get_all_clips().await.unwrap();

    assert!(clips.iter().any(|c| c.id == "test-1"));
}

#[tokio::test]
async fn test_delete_clip() {
    let item = ClipItem {
        id: "test-delete".to_string(),
        r#type: "text".to_string(),
        content: "To delete".to_string(),
        preview: "To delete".to_string(),
        timestamp: Utc::now().timestamp_millis(),
        size: 9,
        ocr_text: None,
        metadata: None,
    };

    add_clip(item).await.unwrap();
    delete_clip("test-delete").await.unwrap();
    let clips = get_all_clips().await.unwrap();

    assert!(!clips.iter().any(|c| c.id == "test-delete"));
}

#[tokio::test]
async fn test_clear_history() {
    let item = ClipItem {
        id: "test-clear".to_string(),
        r#type: "text".to_string(),
        content: "Clear me".to_string(),
        preview: "Clear me".to_string(),
        timestamp: Utc::now().timestamp_millis(),
        size: 8,
        ocr_text: None,
        metadata: None,
    };

    add_clip(item).await.unwrap();
    clear_all_clips().await.unwrap();
    let clips = get_all_clips().await.unwrap();

    assert_eq!(clips.len(), 0);
}

#[tokio::test]
async fn test_rotation() {
    // Add 501 items
    for i in 0..501 {
        let item = ClipItem {
            id: format!("rotation-{}", i),
            r#type: "text".to_string(),
            content: format!("Item {}", i),
            preview: format!("Item {}", i),
            timestamp: Utc::now().timestamp_millis() + i,
            size: 10,
            ocr_text: None,
            metadata: None,
        };
        add_clip(item).await.unwrap();
    }

    let clips = get_all_clips().await.unwrap();
    assert!(clips.len() <= 500);
}