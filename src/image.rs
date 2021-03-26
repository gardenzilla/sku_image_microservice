use chrono::prelude::*;
use packman::VecPackMember;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub trait SkuImageExt {
  fn add_image(
    &mut self,
    file_name: String,
    file_extension: String,
    file_bytes: Vec<u8>,
  ) -> Result<String, String>;
  fn set_cover(&mut self, image_id: String) -> Result<&Self, String>;
  fn swap_images(&mut self, from_id: u32, to_id: u32) -> Result<&Self, String>;
  fn remove_image(&mut self, image_id: String) -> Result<&Self, String>;
  fn get_images(&self) -> &Vec<String>;
  fn get_cover(&self) -> Option<String>;
  fn fix_cover(&mut self);
}

#[derive(Serialize, Deserialize, Clone)]
struct SkuImage {
  pub sku: u32,
  pub cover_image_id: Option<String>,
  pub image_ids: Vec<String>,
}

impl SkuImageExt for SkuImage {
  fn add_image(
    &mut self,
    _file_name: String,
    file_extension: String,
    _file_bytes: Vec<u8>,
  ) -> Result<String, String> {
    // Create new image ID
    let file_id = Uuid::new_v4().to_simple().to_string();
    // Add new image ID to image_ids
    self
      .image_ids
      .push(format!("{}.{}", &file_id, file_extension));
    // Fix Cover
    self.fix_cover();
    // Return Ok image_id
    Ok(file_id)
  }

  fn set_cover(&mut self, image_id: String) -> Result<&Self, String> {
    match self.image_ids.iter().find(|img_id| **img_id == image_id) {
      Some(_) => {
        // Set cover image
        self.cover_image_id = Some(image_id);
        // Fix cover
        self.fix_cover();
        // Return Ok self ref
        Ok(self)
      }
      None => Err("A kért kép ID nem tartozik az adott SKU-hoz.".to_string()),
    }
  }

  fn swap_images(&mut self, from_id: u32, to_id: u32) -> Result<&Self, String> {
    // Check if from_id is a existing position
    if self.image_ids.get(from_id as usize).is_none() {
      return Err("A kezdő pozíció nem létezik a képek között!".to_string());
    }
    // Check if to_id is a existing position
    if self.image_ids.get(to_id as usize).is_none() {
      return Err("A cél pozíció nem létezik a képek között!".to_string());
    }
    // Swap images
    self.image_ids.swap(from_id as usize, to_id as usize);
    // Return Ok self ref
    Ok(self)
  }

  fn remove_image(&mut self, image_id: String) -> Result<&Self, String> {
    let pos = self.image_ids.iter().position(|img_id| *img_id == image_id);
    match pos {
      Some(p) => {
        // Remove IMAGE
        self.image_ids.remove(p);
        // Fix cover
        self.fix_cover();
        // Return Ok self ref
        Ok(self)
      }
      None => Err("A megadott képz nem található, így nem törölhető!".to_string()),
    }
  }

  fn get_images(&self) -> &Vec<String> {
    &self.image_ids
  }

  fn get_cover(&self) -> Option<String> {
    self.cover_image_id.to_owned()
  }

  fn fix_cover(&mut self) {
    // Check if cover ID has set, but ID already removed from image id list;
    // but we have image(s);
    // In this case set None to Cover Image ID
    if let Some(cover_image_id) = &self.cover_image_id {
      if self
        .image_ids
        .iter()
        .find(|img_id| *img_id == cover_image_id)
        .is_none()
      {
        self.cover_image_id = None;
      }
    }
    // Check if cover None, but we have image;
    // In this case set the first image as cover;
    if self.cover_image_id.is_none() {
      if let Some(first_image_id) = self.image_ids.first() {
        self.cover_image_id = Some(first_image_id.clone());
      }
    }
  }
}
