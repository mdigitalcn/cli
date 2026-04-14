define_registry_command!(
    Page,
    "Manage page templates from the registry",
    "Examples:\n\
  mdigitalcn page                             List all pages\n\
  mdigitalcn page add login-page dashboard    Add page templates\n\
  mdigitalcn page list --search auth          Search pages\n\
  mdigitalcn page info settings-page          Show page details\n\
  mdigitalcn page status                      Show installed pages"
);
