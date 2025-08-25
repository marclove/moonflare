use anyhow::Result;
use handlebars::Handlebars;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;
use std::fs;

pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        Self {
            handlebars: Handlebars::new(),
        }
    }
    
    pub fn render_template(&self, template: &str, context: &HashMap<String, Value>) -> Result<String> {
        Ok(self.handlebars.render_template(template, context)?)
    }
    
    pub fn process_template_files(
        &self,
        template_content: &str,
        output_dir: &Path,
        context: &HashMap<String, Value>
    ) -> Result<()> {
        let lines: Vec<&str> = template_content.lines().collect();
        let mut current_file: Option<String> = None;
        let mut current_content = String::new();
        
        for line in lines {
            if line.starts_with("FILE:") {
                // Save previous file if exists
                if let Some(ref file_path) = current_file {
                    let rendered_content = self.render_template(&current_content, context)?;
                    let full_path = output_dir.join(file_path);
                    
                    // Create parent directories
                    if let Some(parent) = full_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    
                    fs::write(full_path, rendered_content)?;
                    current_content.clear();
                }
                
                // Start new file
                current_file = Some(line.strip_prefix("FILE:").unwrap().trim().to_string());
            } else {
                current_content.push_str(line);
                current_content.push('\n');
            }
        }
        
        // Save last file
        if let Some(ref file_path) = current_file {
            let rendered_content = self.render_template(&current_content, context)?;
            let full_path = output_dir.join(file_path);
            
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)?;
            }
            
            fs::write(full_path, rendered_content)?;
        }
        
        Ok(())
    }
}