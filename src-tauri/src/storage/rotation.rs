use crate::storage::repository::{delete_clip, get_all_clips};

pub async fn rotate_if_needed(max_items: usize) -> Result<(), String> {
    let clips = get_all_clips().await?;
    if clips.len() > max_items {
        let to_delete = clips.len() - max_items;
        for clip in clips.iter().rev().take(to_delete) {
            delete_clip(&clip.id).await?;
        }
    }
    Ok(())
}
