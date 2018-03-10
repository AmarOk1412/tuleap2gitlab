# Tuleap 2 Gitlab

This project is a migration script created for one project, the tuleap and gitlab client can be re-used for another project but `issueretriever.rs` is made specifically for one project and should be modified or improved if you want to use it.

The script is 3 parts:
1. Retrieve all issues from a tuleap tracker.
2. Convert tuleap issues in gitlab issues and retrieves all details, comments, and attached files.
3. Post the issue on gitlab.

## Configuration

The configuration must be in a `config.json` file. This is an example of config:

```json
{
  "tuleap_url": "https://your.tracker",
  "tuleap_tracker": 0,
  "gitlab_url": "https://your.git",
  "file_dir": "data",
  "gitlab_token": "yoursecrettoken",
  "assignees": [
    {
      "username":"Your tuleap bot",
      "gitlab_id":"user_id"
    }
  ],
  "projects": [
    {
      "platform":"project1",
      "gitlab_id":"project_id"
    },
    {
      "platform":"project2",
      "gitlab_id":"project_id"
    }
  ],
  "labels": [
    {
      "name":"S - Ordinary",
      "color":"#ffdd99"
    },
    {
      "name":"S - Major",
      "color":"#ffaa00"
    },
    {
      "name":"S - Critical",
      "color":"#1a1100"
    },
    {
      "name":"UI or UX",
      "color":"#0099ff"
    },
    {
      "name":"bug",
      "color":"#FF0000"
    },
    {
      "name":"build",
      "color":"#ff6666"
    },
    {
      "name":"duplicate",
      "color":"#f2f2f2"
    },
    {
      "name":"enhancement",
      "color":"#69D100"
    },
    {
      "name":"feature request",
      "color":"#5CB85C"
    },
    {
      "name":"good first bug",
      "color":"#FFECDB"
    },
    {
      "name":"help wanted",
      "color":"#ffff66"
    },
    {
      "name":"invalid",
      "color":"#d9d9d9"
    },
    {
      "name":"question",
      "color":"#cc0066"
    },
    {
      "name":"refacto",
      "color":"#A8D695"
    },
    {
      "name":"wontfix",
      "color":"#e6e6e6"
    },
    {
      "name":"zombie",
      "color":"#7F8C8D"
    }
  ]
}

```

Note `data` is a directory where the script will download attachments before posting them on gitlab.
