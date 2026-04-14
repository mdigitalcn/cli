define_registry_command!(
    Widget,
    "Manage composed widgets from the registry",
    "Examples:\n\
  mdigitalcn widget                           List all widgets\n\
  mdigitalcn widget add hero-section          Add a widget\n\
  mdigitalcn widget list --category marketing Search by category\n\
  mdigitalcn widget info pricing-table        Show widget details\n\
  mdigitalcn widget status                    Show installed widgets"
);
