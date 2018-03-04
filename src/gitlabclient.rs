pub struct GitlabIssue {
    pub id: String,
    pub title: String,
    pub description: String,
    pub assignee: String,
    pub labels: Vec<String>,
    pub project_url: String,
    pub comments: Vec<String>
}
