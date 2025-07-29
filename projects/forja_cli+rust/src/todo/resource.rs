use url::Url;

#[derive(Debug)]
pub(crate) enum Resource {
    GitHubIssue {
        owner: String,
        repository: String,
        id: u32,
    },
    Url(Url),
}

impl Resource {
    pub fn try_find_resource(text: impl AsRef<str>) -> Option<Resource> {
        let url = match Url::parse(text.as_ref()) {
            Ok(url) => url,
            Err(_) => return None,
        };

        let host = match url.host_str() {
            Some(host) => host,

            // Allowing "host-less" URLs creates lots of unwanted resources
            None => return None,
        };

        if let Some((owner, repository, id)) = Self::try_parse_github_issue(host, url.path()) {
            return Some(Self::GitHubIssue {
                owner,
                repository,
                id,
            });
        }

        Some(Resource::Url(url))
    }

    fn try_parse_github_issue(host: &str, path: &str) -> Option<(String, String, u32)> {
        if host != "github.com" {
            return None;
        }

        let path_parts: Vec<&str> = path.trim_start_matches("/").split("/").collect();

        if path_parts.len() != 4 {
            return None;
        }

        if path_parts[2] != "issues" {
            return None;
        }

        let id = match path_parts[3].parse::<u32>() {
            Ok(id) => id,
            Err(_) => {
                return None;
            }
        };

        Some((path_parts[0].to_owned(), path_parts[1].to_owned(), id))
    }
}
