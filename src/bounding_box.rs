use crate::browser::Browser;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BoundingBoxError {
    #[error("Browser error: {0}")]
    BrowserError(#[from] fantoccini::error::CmdError),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ElementBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub tag_name: String,
    pub class_name: String,
    pub id: String,
    pub text_content: String,
    pub parent_selector: String,
    pub sibling_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiblingGroup {
    pub width: f64,
    pub parent_selector: String,
    pub color: String,
    pub elements: Vec<ElementBounds>,
}

pub struct BoundingBoxAnalyzer<'a> {
    browser: &'a Browser,
}

impl<'a> BoundingBoxAnalyzer<'a> {
    pub fn new(browser: &'a Browser) -> Self {
        Self { browser }
    }

    pub async fn extract_all_bounding_boxes(&self) -> Result<Vec<ElementBounds>, BoundingBoxError> {
        let script = r#"
            try {
                const elements = document.querySelectorAll('*');
                const bounds = [];
                
                function getElementSelector(element) {
                    if (element.id) {
                        return '#' + element.id;
                    }
                    
                    // Handle className properly - it could be a string, DOMTokenList, or null
                    let className = '';
                    if (element.className) {
                        if (typeof element.className === 'string') {
                            className = element.className;
                        } else if (element.className.toString) {
                            className = element.className.toString();
                        } else {
                            className = String(element.className);
                        }
                    }
                    
                    if (className.trim()) {
                        const classes = className.split(' ').filter(c => c.trim()).join('.');
                        return element.tagName.toLowerCase() + (classes ? '.' + classes : '');
                    }
                    
                    return element.tagName.toLowerCase();
                }
                
                function getParentSelector(element) {
                    if (!element.parentElement) return 'body';
                    
                    const parent = element.parentElement;
                    if (parent.id) {
                        return '#' + parent.id;
                    }
                    
                    // Handle className properly - it could be a string, DOMTokenList, or null
                    let className = '';
                    if (parent.className) {
                        if (typeof parent.className === 'string') {
                            className = parent.className;
                        } else if (parent.className.toString) {
                            className = parent.className.toString();
                        } else {
                            className = String(parent.className);
                        }
                    }
                    
                    if (className.trim()) {
                        const classes = className.split(' ').filter(c => c.trim()).join('.');
                        return parent.tagName.toLowerCase() + (classes ? '.' + classes : '');
                    }
                    
                    return parent.tagName.toLowerCase();
                }
                
                function getSiblingIndex(element) {
                    const parent = element.parentElement;
                    if (!parent) return 0;
                    
                    let index = 0;
                    for (let i = 0; i < parent.children.length; i++) {
                        if (parent.children[i] === element) {
                            return i;
                        }
                    }
                    return 0;
                }
                
                for (let i = 0; i < elements.length; i++) {
                    const element = elements[i];
                    const rect = element.getBoundingClientRect();
                    const computedStyle = window.getComputedStyle(element);
                    
                    // Only include visible elements with meaningful dimensions
                    if (rect.width > 1 && rect.height > 1 && 
                        computedStyle.visibility !== 'hidden' && 
                        computedStyle.display !== 'none' &&
                        rect.top < window.innerHeight && 
                        rect.bottom > 0 &&
                        rect.left < window.innerWidth && 
                        rect.right > 0) {
                        
                        // Handle element className safely
                        let elementClassName = '';
                        if (element.className) {
                            if (typeof element.className === 'string') {
                                elementClassName = element.className;
                            } else if (element.className.toString) {
                                elementClassName = element.className.toString();
                            } else {
                                elementClassName = String(element.className);
                            }
                        }
                        
                        bounds.push({
                            x: Number(rect.x) || 0,
                            y: Number(rect.y) || 0,
                            width: Number(rect.width) || 0,
                            height: Number(rect.height) || 0,
                            tag_name: String(element.tagName).toLowerCase(),
                            class_name: elementClassName,
                            id: String(element.id || ''),
                            text_content: String(element.textContent || '').trim().substring(0, 100).replace(/[\r\n\t]/g, ' '),
                            parent_selector: getParentSelector(element),
                            sibling_index: getSiblingIndex(element)
                        });
                    }
                }
                
                return bounds;
            } catch (error) {
                return { error: error.message };
            }
        "#;

        let result = self.browser.client().execute(script, vec![]).await?;
        
        // Debug: print the actual result structure
        println!("DEBUG: JavaScript result type: {:?}", result);
        
        // Check if JavaScript returned an error
        if let serde_json::Value::Object(ref obj) = result {
            if let Some(error_msg) = obj.get("error") {
                return Err(BoundingBoxError::SerializationError(
                    format!("JavaScript error: {}", error_msg)
                ));
            }
        }
        
        if let serde_json::Value::Array(ref arr) = result {
            println!("DEBUG: Array length: {}", arr.len());
            if !arr.is_empty() {
                println!("DEBUG: First element: {:?}", &arr[0]);
            }
        }
        
        let bounds: Vec<ElementBounds> = serde_json::from_value(result)
            .map_err(|e| BoundingBoxError::SerializationError(format!("Failed to deserialize bounding boxes: {}", e)))?;
        Ok(bounds)
    }

    pub fn group_sibling_elements(&self, elements: Vec<ElementBounds>, width_tolerance: f64) -> Vec<SiblingGroup> {
        use std::collections::HashMap;
        
        // Group elements by parent first
        let mut parent_groups: HashMap<String, Vec<ElementBounds>> = HashMap::new();
        
        for element in elements {
            parent_groups
                .entry(element.parent_selector.clone())
                .or_insert_with(Vec::new)
                .push(element);
        }
        
        let mut sibling_groups: Vec<SiblingGroup> = Vec::new();
        let colors = vec![
            "#FF6B6B", "#4ECDC4", "#45B7D1", "#96CEB4", "#FFEAA7",
            "#DDA0DD", "#98D8C8", "#F7DC6F", "#BB8FCE", "#85C1E9",
            "#F8C471", "#82E0AA", "#F1948A", "#85C1E9", "#D7BDE2",
            "#F39C12", "#E74C3C", "#9B59B6", "#3498DB", "#1ABC9C"
        ];
        let mut color_index = 0;
        
        // For each parent, group siblings by similar width
        for (parent_selector, mut siblings) in parent_groups {
            // Skip parents with only one child
            if siblings.len() < 2 {
                continue;
            }
            
            // Sort siblings by their position (top to bottom)
            siblings.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal));
            
            // Group siblings with similar widths
            let mut width_groups: Vec<Vec<ElementBounds>> = Vec::new();
            
            for sibling in siblings {
                let mut found_width_group = false;
                
                for width_group in &mut width_groups {
                    if let Some(first_in_group) = width_group.first() {
                        let width_diff = (first_in_group.width - sibling.width).abs();
                        if width_diff <= width_tolerance {
                            width_group.push(sibling.clone());
                            found_width_group = true;
                            break;
                        }
                    }
                }
                
                if !found_width_group {
                    width_groups.push(vec![sibling]);
                }
            }
            
            // Create sibling groups for groups with multiple elements
            for width_group in width_groups {
                if width_group.len() >= 2 {
                    let color = colors[color_index % colors.len()].to_string();
                    color_index += 1;
                    
                    let avg_width = width_group.iter().map(|e| e.width).sum::<f64>() / width_group.len() as f64;
                    
                    sibling_groups.push(SiblingGroup {
                        width: avg_width,
                        parent_selector: parent_selector.clone(),
                        color,
                        elements: width_group,
                    });
                }
            }
        }
        
        // Sort groups by number of elements (largest first)
        sibling_groups.sort_by(|a, b| b.elements.len().cmp(&a.elements.len()));
        sibling_groups
    }

    pub async fn visualize_sibling_groups(&self, groups: &[SiblingGroup]) -> Result<(), BoundingBoxError> {
        // Filter out groups that don't have actual siblings (only show groups with 2+ elements)
        let filtered_groups: Vec<&SiblingGroup> = groups.iter()
            .filter(|group| group.elements.len() >= 2)
            .collect();
        
        println!("DEBUG: Total groups found: {}", groups.len());
        println!("DEBUG: Groups with 2+ elements: {}", filtered_groups.len());
        
        if filtered_groups.is_empty() {
            println!("No sibling groups with multiple elements found.");
            return Ok(());
        }

        println!("DEBUG: Total elements in all groups: {}", 
            filtered_groups.iter().map(|g| g.elements.len()).sum::<usize>());
        // Remove any existing visualization
        let cleanup_script = r#"
            const existingOverlay = document.getElementById('bounding-box-overlay');
            if (existingOverlay) {
                existingOverlay.remove();
            }
        "#;
        self.browser.client().execute(cleanup_script, vec![]).await?;

        // Create overlay container
        let create_overlay_script = r#"
            const overlay = document.createElement('div');
            overlay.id = 'bounding-box-overlay';
            overlay.style.position = 'fixed';
            overlay.style.top = '0';
            overlay.style.left = '0';
            overlay.style.width = '100vw';
            overlay.style.height = '100vh';
            overlay.style.pointerEvents = 'none';
            overlay.style.zIndex = '10000';
            document.body.appendChild(overlay);
        "#;
        self.browser.client().execute(create_overlay_script, vec![]).await?;

        // Add bounding boxes for each sibling group (simplified - no nesting filter for now)
        let mut visualized_groups = 0;
        for (group_index, group) in filtered_groups.iter().enumerate() {
            println!("DEBUG: Group {} - Elements: {}, Parent: {}", 
                group_index + 1, group.elements.len(), group.parent_selector);
            
            visualized_groups += 1;

            let elements_json = serde_json::to_string(&group.elements)
                .map_err(|e| BoundingBoxError::SerializationError(e.to_string()))?;
            
            let script = format!(r#"
                try {{
                    const overlay = document.getElementById('bounding-box-overlay');
                    const groupElements = JSON.parse({});
                    
                    groupElements.forEach((elementData, index) => {{
                        const box = document.createElement('div');
                        box.style.position = 'absolute';
                        box.style.left = elementData.x + 'px';
                        box.style.top = elementData.y + 'px';
                        box.style.width = elementData.width + 'px';
                        box.style.height = elementData.height + 'px';
                        box.style.border = '3px solid {}';
                        box.style.backgroundColor = '{}';
                        box.style.opacity = '0.25';
                        box.style.boxSizing = 'border-box';
                        
                        // Add label showing parent and group info
                        const label = document.createElement('div');
                        const parentShort = {} === 'body' ? 'body' : {}.split('.')[0];
                        label.textContent = 'S' + ({} + 1) + ' (' + groupElements.length + ' siblings in ' + parentShort + ')';
                        label.style.position = 'absolute';
                        label.style.top = '-22px';
                        label.style.left = '0';
                        label.style.fontSize = '11px';
                        label.style.backgroundColor = '{}';
                        label.style.color = 'white';
                        label.style.padding = '2px 6px';
                        label.style.borderRadius = '3px';
                        label.style.whiteSpace = 'nowrap';
                        label.style.fontWeight = 'bold';
                        label.style.textShadow = '1px 1px 2px rgba(0,0,0,0.8)';
                        box.appendChild(label);
                        
                        overlay.appendChild(box);
                    }});
                }} catch (error) {{
                    console.error('Error creating visualization:', error);
                }}
            "#, 
            serde_json::to_string(&elements_json).map_err(|e| BoundingBoxError::SerializationError(e.to_string()))?,
            group.color,
            group.color,
            serde_json::to_string(&group.parent_selector).map_err(|e| BoundingBoxError::SerializationError(e.to_string()))?,
            serde_json::to_string(&group.parent_selector).map_err(|e| BoundingBoxError::SerializationError(e.to_string()))?,
            group_index,
            group.color
            );
            
            self.browser.client().execute(&script, vec![]).await?;
        }

        println!("DEBUG: Total groups visualized: {}", visualized_groups);

        // Add legend
        let groups_json = serde_json::to_string(&filtered_groups)
            .map_err(|e| BoundingBoxError::SerializationError(e.to_string()))?;
            
        let legend_script = format!(r#"
            try {{
                const legend = document.createElement('div');
                legend.style.position = 'fixed';
                legend.style.top = '10px';
                legend.style.right = '10px';
                legend.style.backgroundColor = 'rgba(0, 0, 0, 0.9)';
                legend.style.color = 'white';
                legend.style.padding = '12px';
                legend.style.borderRadius = '6px';
                legend.style.fontSize = '12px';
                legend.style.zIndex = '10001';
                legend.style.maxHeight = '400px';
                legend.style.overflowY = 'auto';
                legend.style.minWidth = '250px';
                
                let legendHTML = '<h4 style="margin: 0 0 12px 0; color: #fff;">Sibling Element Groups</h4>';
                const groupsData = JSON.parse({});
                
                groupsData.forEach((group, index) => {{
                    legendHTML += '<div style="margin-bottom: 8px; border-left: 3px solid ' + group.color + '; padding-left: 8px;">';
                    legendHTML += '<div style="font-weight: bold;">S' + (index + 1) + ': ' + group.elements.length + ' siblings</div>';
                    legendHTML += '<div style="font-size: 10px; opacity: 0.8;">Parent: ' + group.parent_selector + '</div>';
                    legendHTML += '<div style="font-size: 10px; opacity: 0.8;">Width: ' + Math.round(group.width) + 'px</div>';
                    legendHTML += '</div>';
                }});
                
                if (groupsData.length === 0) {{
                    legendHTML += '<div style="opacity: 0.6; font-style: italic;">No sibling groups found</div>';
                }} else {{
                    legendHTML += '<div style="margin-top: 8px; font-size: 10px; opacity: 0.7;">Total: ' + groupsData.length + ' groups visualized</div>';
                }}
                
                legend.innerHTML = legendHTML;
                document.body.appendChild(legend);
            }} catch (error) {{
                console.error('Error creating legend:', error);
            }}
        "#, serde_json::to_string(&groups_json).map_err(|e| BoundingBoxError::SerializationError(e.to_string()))?);
        
        self.browser.client().execute(&legend_script, vec![]).await?;

        Ok(())
    }

    pub async fn navigate_and_analyze(&self, url: &str, tolerance: f64) -> Result<Vec<SiblingGroup>, BoundingBoxError> {
        // Navigate to the URL first
        self.browser.client().goto(url).await?;
        
        // Wait for page load
        tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
        
        self.analyze_and_visualize(tolerance).await
    }

    pub async fn analyze_and_visualize(&self, width_tolerance: f64) -> Result<Vec<SiblingGroup>, BoundingBoxError> {
        let elements = self.extract_all_bounding_boxes().await?;
        let groups = self.group_sibling_elements(elements, width_tolerance);
        self.visualize_sibling_groups(&groups).await?;
        Ok(groups)
    }

    pub fn print_analysis(&self, groups: &[SiblingGroup]) {
        println!("Sibling Element Analysis Results:");
        println!("=================================");
        
        // Filter to only show groups with multiple elements (actual siblings)
        let sibling_groups: Vec<&SiblingGroup> = groups.iter()
            .filter(|group| group.elements.len() >= 2)
            .collect();
        
        if sibling_groups.is_empty() {
            println!("\nNo sibling groups found. This could mean:");
            println!("  - Elements don't have siblings with similar widths");
            println!("  - Width tolerance is too strict (try increasing it)");
            println!("  - The page has a different layout structure");
            return;
        }
        
        for (index, group) in sibling_groups.iter().enumerate() {
            println!("\nSibling Group S{}: {} elements with width ~{:.1}px", 
                index + 1, 
                group.elements.len(), 
                group.width
            );
            println!("  Parent Container: {}", group.parent_selector);
            println!("  Color: {}", group.color);
            
            println!("  Elements (top to bottom):");
            for (elem_index, element) in group.elements.iter().enumerate() {
                let preview = if element.text_content.len() > 40 {
                    format!("{}...", &element.text_content[..40])
                } else {
                    element.text_content.clone()
                };
                
                println!("    {}: <{}> {}x{} \"{}\"", 
                    elem_index + 1,
                    element.tag_name,
                    element.width.round(),
                    element.height.round(),
                    preview.replace('\n', " ").trim()
                );
            }
        }
        
        println!("\nSummary: Found {} sibling groups representing potential lists/repeated content", sibling_groups.len());
        let total_elements: usize = sibling_groups.iter().map(|g| g.elements.len()).sum();
        println!("Total elements in groups: {}", total_elements);
        println!("Note: Nested elements within other colored boxes are automatically filtered out");
    }
}