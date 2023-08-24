use std::str::FromStr;

use globset::{GlobBuilder, GlobMatcher};
use tracing::debug;
use url::Url;

/// [scheme://]hostname[/path][?query][#fragment]
/// [*://]**[/**][?**][#*]
// TODO: replace hostname with user:password@hostname:port
#[derive(Debug, PartialEq)]
pub struct UrlMatcher {
    scheme: String,
    user: String,
    password: String,
    hostname: String,
    port: String,
    path: String,
    query: String,
    fragment: String,
}

#[derive(Clone, Debug)]
pub struct UrlGlobMatcher {
    scheme: GlobMatcher,
    user: GlobMatcher,
    password: GlobMatcher,
    hostname: GlobMatcher,
    port: GlobMatcher,
    path: GlobMatcher,
    query: GlobMatcher,
    fragment: GlobMatcher,
}

impl UrlGlobMatcher {
    fn from_url_matcher(url_matcher: &UrlMatcher) -> Self {
        let scheme_matcher = Self::str_to_glob(url_matcher.scheme.as_str(), "scheme");

        let user_matcher = Self::str_to_glob(url_matcher.user.as_str(), "user");
        let password_matcher = Self::str_to_glob(url_matcher.password.as_str(), "password");

        // "my.path.**" -> "my/path/**"
        let hostname_with_slashes = url_matcher.hostname.replace(".", "/");
        let hostname_matcher = Self::str_to_glob(hostname_with_slashes.as_str(), "hostname");

        let port_matcher = Self::str_to_glob(url_matcher.password.as_str(), "port");

        let path_matcher = Self::str_to_glob(url_matcher.path.as_str(), "path");

        // "name=ferret&color=purple" -> "name=ferret/color=purple"
        let query_with_slashes = url_matcher.query.replace("&", "/");
        let query_matcher = Self::str_to_glob(query_with_slashes.as_str(), "query");
        let fragment_matcher = Self::str_to_glob(url_matcher.fragment.as_str(), "fragment");

        Self {
            scheme: scheme_matcher,
            user: user_matcher,
            password: password_matcher,
            hostname: hostname_matcher,
            port: port_matcher,
            path: path_matcher,
            query: query_matcher,
            fragment: fragment_matcher,
        }
    }

    fn str_to_glob(pattern: &str, name: &str) -> GlobMatcher {
        let glob = GlobBuilder::new(pattern)
            .literal_separator(true)
            .case_insensitive(true)
            .build()
            .expect(format!("illegal pattern for {name}").as_str());

        let glob_matcher = glob.compile_matcher();
        return glob_matcher;
    }

    fn to_target_url(&self, url: &Url) -> TargetUrl {
        let scheme = url.scheme();
        let host = url
            .host_str()
            .unwrap_or_else(|| panic!("no host found from url: {}", url.as_str()));

        let user = url.username();
        let password = url.password().unwrap_or("");
        let port = url.port().map(|x| x.to_string()).unwrap_or("".to_string());
        let path = url.path();
        let query = url.query().unwrap_or("");
        let fragment = url.fragment().unwrap_or("");

        return TargetUrl {
            scheme: scheme.to_string(),
            user: user.to_string(),
            password: password.to_string(),
            hostname: host.to_string(),
            port: port.to_string(),
            path: path.to_string(),
            query: query.to_string(),
            fragment: fragment.to_string(),
        };
    }

    pub fn url_str_matches(&self, url_str: &str) -> bool {
        let url = Url::from_str(url_str).unwrap_or_else(|_| panic!("not a valid url: {}", url_str));

        return self.url_matches(&url);
    }

    pub fn url_matches(&self, url: &Url) -> bool {
        let target_url = self.to_target_url(url);

        //self.scheme.is_match_candidate()
        let scheme_matches = self.scheme.is_match(target_url.scheme);

        let username_matches = self.user.is_match(target_url.user);
        let password_matches = self.password.is_match(target_url.password);

        let hostname_matches = self.hostname_matches(target_url.hostname.as_str());
        let path_matches = self.path.is_match(target_url.path);
        let port_matches = self.port.is_match(target_url.port);

        let target_query_with_slashes = target_url.query.replace("&", "/");
        let query_matches = self.query.is_match(target_query_with_slashes);
        let fragment_matches = self.fragment.is_match(target_url.fragment);

        return scheme_matches
            && username_matches
            && password_matches
            && hostname_matches
            && path_matches
            && port_matches
            && query_matches
            && fragment_matches;
    }

    fn hostname_matches(&self, target_hostname: &str) -> bool {
        let target_hostname_with_slashes = target_hostname.replace(".", "/");
        return self.hostname.is_match(target_hostname_with_slashes);
    }
}

impl UrlMatcher {
    pub fn to_glob_matcher(&self) -> UrlGlobMatcher {
        UrlGlobMatcher::from_url_matcher(self)
    }
}

struct TargetUrl {
    scheme: String,
    user: String,
    password: String,
    hostname: String,
    port: String,
    path: String,
    query: String,
    fragment: String,
}

// TODO: parse from the end to beginning
fn extract_part_matchers(full_rule: &str) -> UrlMatcher {
    // full_rule = https://hostname/path?query#fragment
    //assert_eq!(s.find("pard"), Some(17));
    let scheme_end_index = full_rule.find("://").expect("no scheme with :// suffix");
    // https
    let scheme_pattern = &full_rule[..scheme_end_index];
    // hostname/path?query#fragment
    let after_scheme = &full_rule[scheme_end_index + 3..];

    let after_hostname_index = after_scheme.find("/").expect("no hostname with / suffix");
    // hostname
    let hostname_pattern = &after_scheme[..after_hostname_index];
    // /path?query#fragment
    let after_hostname = &after_scheme[after_hostname_index..];

    let after_path_index = after_hostname.find("?").expect("no path with ? suffix");
    // /path
    let path_pattern = &after_hostname[..after_path_index];
    // query#fragment
    let after_path = &after_hostname[after_path_index + 1..];

    let after_query_index = after_path.find("#").expect("no query with # suffix");
    // query
    let query_pattern = &after_path[..after_query_index];
    // fragment
    let after_query = &after_path[after_query_index + 1..];
    // fragment
    let fragment_pattern = &after_query;

    return UrlMatcher {
        scheme: scheme_pattern.to_string(),
        user: "".to_string(),
        password: "".to_string(),
        hostname: hostname_pattern.to_string(),
        port: "".to_string(),
        path: path_pattern.to_string(),
        query: query_pattern.to_string(),
        fragment: fragment_pattern.to_string(),
    };
}

pub fn to_url_matcher(rule: &str) -> UrlMatcher {
    let full_rule = transform_to_full_match(rule);
    let url_matcher = extract_part_matchers(&full_rule);
    debug!("parsed url matcher: {:?}", url_matcher);
    return url_matcher;
}

fn transform_to_full_match(rule: &str) -> String {
    let rule = add_scheme_matcher(rule);
    // hostname matcher is mandatory
    let rule = add_path_matcher(rule.as_str());
    let rule = add_query_matcher(rule.as_str());
    let rule = add_fragment_matcher(rule.as_str());
    return rule;
}

fn add_scheme_matcher(rule: &str) -> String {
    return if !rule.contains("://") {
        String::from("*://") + rule
    } else {
        rule.to_string()
    };
}

// requires scheme matcher to be already present
fn add_path_matcher(rule: &str) -> String {
    let scheme_end_index = rule.find("://").expect("no scheme with :// suffix");
    // hostname/path?query#fragment
    let after_scheme = &rule[scheme_end_index + 3..];

    return if !after_scheme.contains("/") {
        rule.to_string() + "/**" // path can have multiple parts
    } else {
        rule.to_string()
    };
}

fn add_query_matcher(rule: &str) -> String {
    return if !rule.contains("?") {
        rule.to_string() + "?**" // query can have multiple parameters
    } else {
        rule.to_string()
    };
}

fn add_fragment_matcher(rule: &str) -> String {
    return if !rule.contains("#") {
        rule.to_string() + "#*" // fragment has only one part
    } else {
        rule.to_string()
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_hostname_to_full_match() {
        assert_eq!(transform_to_full_match("example.com"), "*://example.com/**?**#*");
    }

    #[test]
    fn test_extract_part_matchers() {
        assert_eq!(
            extract_part_matchers("*://example.com/?#"),
            UrlMatcher {
                scheme: "*".to_string(),
                user: "".to_string(),
                password: "".to_string(),
                hostname: "example.com".to_string(),
                port: "".to_string(),
                path: "/".to_string(),
                query: "".to_string(),
                fragment: "".to_string(),
            }
        );
    }

    #[test]
    fn test_to_url_matcher_parses_full_match() {
        assert_eq!(
            to_url_matcher("*://example.com/?#"),
            UrlMatcher {
                scheme: "*".to_string(),
                user: "".to_string(),
                password: "".to_string(),
                hostname: "example.com".to_string(),
                port: "".to_string(),
                path: "/".to_string(),
                query: "".to_string(),
                fragment: "".to_string(),
            }
        );
    }
    #[test]
    fn test_to_url_matcher_fills_scheme_with_wildcard() {
        assert_eq!(
            to_url_matcher("example.com/?#"),
            UrlMatcher {
                scheme: "*".to_string(),
                user: "".to_string(),
                password: "".to_string(),
                hostname: "example.com".to_string(),
                port: "".to_string(),
                path: "/".to_string(),
                query: "".to_string(),
                fragment: "".to_string(),
            }
        );
    }

    #[test]
    fn test_to_url_matcher_fills_scheme_fragment_with_wildcard() {
        assert_eq!(
            to_url_matcher("example.com/?"),
            UrlMatcher {
                scheme: "*".to_string(),
                user: "".to_string(),
                password: "".to_string(),
                hostname: "example.com".to_string(),
                port: "".to_string(),
                path: "/".to_string(),
                query: "".to_string(),
                fragment: "*".to_string(),
            }
        );
    }

    #[test]
    fn test_to_url_matcher_fills_scheme_query_fragment_with_wildcard() {
        assert_eq!(
            to_url_matcher("example.com/"),
            UrlMatcher {
                scheme: "*".to_string(),
                user: "".to_string(),
                password: "".to_string(),
                hostname: "example.com".to_string(),
                port: "".to_string(),
                path: "/".to_string(),
                query: "**".to_string(),
                fragment: "*".to_string(),
            }
        );
    }

    #[test]
    fn test_to_url_matcher_fills_scheme_path_query_fragment_with_wildcard() {
        assert_eq!(
            to_url_matcher("example.com"),
            UrlMatcher {
                scheme: "*".to_string(),
                user: "".to_string(),
                password: "".to_string(),
                hostname: "example.com".to_string(),
                port: "".to_string(),
                path: "/**".to_string(),
                query: "**".to_string(),
                fragment: "*".to_string(),
            }
        );
    }

    #[test]
    fn test_to_url_matcher_fills_path_and_query_and_fragment_with_wildcard() {
        assert_eq!(
            to_url_matcher("*://example.com"),
            UrlMatcher {
                scheme: "*".to_string(),
                user: "".to_string(),
                password: "".to_string(),
                hostname: "example.com".to_string(),
                port: "".to_string(),
                path: "/**".to_string(),
                query: "**".to_string(),
                fragment: "*".to_string(),
            }
        );
    }

    #[test]
    fn test_to_url_matcher_examples() {
        assert_eq!(
            to_url_matcher("app.company.xyz/v2/**"),
            UrlMatcher {
                scheme: "*".to_string(),
                user: "".to_string(),
                password: "".to_string(),
                hostname: "app.company.xyz".to_string(),
                port: "".to_string(),
                path: "/v2/**".to_string(),
                query: "**".to_string(),
                fragment: "*".to_string(),
            }
        );
    }

    #[test]
    fn test_url_matches_example_1() {
        let url_matcher = to_url_matcher("app.company.xyz/v2/**");
        let url_glob_matcher = url_matcher.to_glob_matcher();
        let matches =
            url_glob_matcher.url_str_matches("https://app.company.xyz/v2/matches/everything");
        assert_eq!(matches, true);
    }

    #[test]
    fn test_url_matches_matches_path_with_two_asterisk() {
        let url_matcher = to_url_matcher("beginning.**/**");
        let url_glob_matcher = url_matcher.to_glob_matcher();
        let matches = url_glob_matcher
            .url_str_matches("https://beginning.of.something.great/v2/matches/everything");
        assert_eq!(matches, true);
    }

    #[test]
    fn test_url_matches_doesnt_match_path_with_one_asterisk() {
        let url_matcher = to_url_matcher("beginning.**/*");
        let url_glob_matcher = url_matcher.to_glob_matcher();
        let matches = url_glob_matcher
            .url_str_matches("https://beginning.of.something.great/v2/matches/everything");
        assert_eq!(matches, false);
    }

    #[test]
    fn test_url_matches_doesnt_matches_domain_with_two_asterisks() {
        assert_eq!(
            to_url_matcher("beginning.**")
                .to_glob_matcher()
                .url_str_matches("https://beginning.of.something.great"),
            true
        );

        assert_eq!(
            to_url_matcher("beginning.**.great")
                .to_glob_matcher()
                .url_str_matches("https://beginning.of.something.great"),
            true
        );

        assert_eq!(
            to_url_matcher("beginning.**.notgreat")
                .to_glob_matcher()
                .url_str_matches("https://beginning.of.something.great"),
            false
        );
    }

    #[test]
    fn test_url_matches_doesnt_match_domain_with_one_asterisk() {
        let url_matcher = to_url_matcher("beginning.*/**");
        let url_glob_matcher = url_matcher.to_glob_matcher();
        let matches = url_glob_matcher
            .url_str_matches("https://beginning.of.something.great/v2/matches/everything");
        assert_eq!(matches, false);
    }
}
