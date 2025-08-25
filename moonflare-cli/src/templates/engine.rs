use anyhow::Result;
use handlebars::Handlebars;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use convert_case::{Case, Casing};

pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();
        
        // Register helper for uppercase conversion (SCREAMING_SNAKE_CASE)
        handlebars.register_helper("upper", Box::new(|h: &handlebars::Helper, _: &handlebars::Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output| -> handlebars::HelperResult {
            let param = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
            let upper_case = param.to_case(Case::ScreamingSnake);
            out.write(&upper_case)?;
            Ok(())
        }));
        
        // Register helper for title case conversion (PascalCase)
        handlebars.register_helper("title", Box::new(|h: &handlebars::Helper, _: &handlebars::Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output| -> handlebars::HelperResult {
            let param = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
            let pascal_case = param.to_case(Case::Pascal);
            out.write(&pascal_case)?;
            Ok(())
        }));
        
        Self {
            handlebars,
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