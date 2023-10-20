use std::fmt::Display;

use crate::pmap::*;

#[derive(Debug, PartialEq)]
pub struct PMapCategory {
    pub name: String,
    pub total_size_in_kibibyte: u64,
    pub pages: Vec<PMap>,
}

impl PMapCategory {
    fn new(name: String) -> Self {
        Self {
            name,
            total_size_in_kibibyte: 0,
            pages: Vec::new(),
        }
    }

    fn add_page(&mut self, page: PMap) {
        self.total_size_in_kibibyte += page.size_in_kibibyte;
        self.pages.push(page);
    }

    pub fn get_categories_from_memory_pages(
        memory_pages: PMapVec,
        get_custom_category_name: &dyn Fn(MappingKind) -> String)
        -> Result<PMapCategoryVec, String> {

        let mut categories: PMapCategoryVec = PMapCategoryVec(Vec::new());
        for page in memory_pages.0{
            let category_name: Result<String, String> = match page.mapping_kind {
                MappingKind::File(_) => Ok(get_custom_category_name(page.mapping_kind.clone())),
                MappingKind::AnonymousPrivate(None) => Ok("Anonymous".to_string()),
                MappingKind::AnonymousPrivate(Some(_)) => Ok(get_custom_category_name(page.mapping_kind.clone())),
                MappingKind::AnonymousShared(None) => Ok("Anonymous".to_string()),
                MappingKind::AnonymousShared(Some(_)) => Ok(get_custom_category_name(page.mapping_kind.clone())),
                MappingKind::Heap => Ok("[heap]".to_string()),
                MappingKind::Stack => Ok("[stack]".to_string()),
                MappingKind::VirtualVariables => Ok("[vvar]".to_string()),
                MappingKind::VirtualDynamicSharedObject => Ok("[vdso]".to_string()),
                MappingKind::VirtualSystemCall => Ok("[vsyscall]".to_string()),
            };
            let category_name = category_name?;

            let category = match categories.0.iter_mut().find(|category| category.name == category_name) {
                Some(category) => category,
                None => {
                    let new_category = PMapCategory::new(category_name);
                    categories.0.push(new_category);
                    categories.0.last_mut().unwrap()
                }
            };
            category.add_page(page);
        }

        categories.0.sort_by(|a, b| b.total_size_in_kibibyte.cmp(&a.total_size_in_kibibyte));
        Ok(categories)
    }
}


impl Display for PMapCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format!("| {:56} | {:10} | {:15} |", self.name, self.total_size_in_kibibyte, self.pages.len()).fmt(f)
    }
}

pub struct PMapCategoryVec(pub Vec<PMapCategory>);

impl Display for PMapCategoryVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut total_size: u64 = 0;
        let mut total_pages: u64 = 0;
        format!("|----------------------------------------------------------|------------|-----------------|\n").fmt(f)?;
        format!("| {:56} | {:10} | {:15} |\n", "Category", "Size [KiB]", "#Memory Pages").fmt(f)?;
        format!("|----------------------------------------------------------|------------|-----------------|\n").fmt(f)?;

        for category in &self.0[0..self.0.len() - 1] {
            category.fmt(f)?;
            writeln!(f)?;
            total_size += category.total_size_in_kibibyte;
            total_pages += category.pages.len() as u64;
        }
        format!("|----------------------------------------------------------|------------|-----------------|\n").fmt(f)?;
        format!("| {:56} | {:10} | {:15} |\n","", total_size, total_pages).fmt(f)?;
        format!("|----------------------------------------------------------|------------|-----------------|\n").fmt(f)?;
        writeln!(f)?;

        Ok(())
    }
}
