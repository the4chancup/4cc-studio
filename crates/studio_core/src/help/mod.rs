//! Help chapter types. The help window is assembled from the tools: every registered tool
//! contributes one `HelpSection` through `StudioTool::help()`, and `studio_core` contributes the
//! core chapters (core plan, "Help window"). Search, rendering and export come with the GUI.

/// One tool's chapter of the help window: a title and its topics, in reading order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelpSection {
    /// Chapter title, normally the tool's label.
    pub title: &'static str,
    /// Topics in order; the numbered file names in the tool's `help/` folder fix it.
    pub topics: Vec<HelpTopic>,
}

/// One topic of a chapter: a Markdown body embedded with `include_str!`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelpTopic {
    /// Link target within the chapter (`exports` in `[text](team-compiler#exports)`).
    pub slug: &'static str,
    /// Title shown in the chapter tree.
    pub title: &'static str,
    /// The Markdown body.
    pub body: &'static str,
}

/// Where a link, a search result or a message-id click opens the help window.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HelpTarget {
    /// Tool id owning the chapter (`team-compiler`), or the core chapter's id.
    pub tool: String,
    /// Topic slug within the chapter; `None` opens the chapter's first topic.
    pub topic: Option<String>,
    /// Anchor inside the topic, such as a message code in the generated "Messages" topic.
    pub anchor: Option<String>,
}
