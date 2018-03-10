# Tuleap 2 Gitlab

This project is a migration script created for one project, the tuleap and gitlab client can be re-used for another project but `issueretriever.rs` is made specifically for one project and should be modified or improved if you want to use it.

The script is 3 parts:
1. Retrieve all issues from a tuleap tracker.
2. Convert tuleap issues in gitlab issues and retrieves all details, comments, and attached files.
3. Post the issue on gitlab.
