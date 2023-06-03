use tracing::info;
use url::Url;

pub fn convert_slack_uri(profile_team_id: &str, profile_team_domain: &str, url: &Url) -> String {
    let unknown = format!("slack://channel?team={}", profile_team_id);

    let url_host_str = (&url.host_str().unwrap()).to_string();

    let path_segments_maybe = url.path_segments();
    let url_path_segments_maybe = path_segments_maybe.map(|c| {
        let vec = c.map(str::to_string).collect::<Vec<_>>();
        vec
    });

    // https://slack.com/help/articles/221769328-Locate-your-Slack-URL
    // https://api.slack.com/reference/deep-linking#supported_URIs
    return if url_host_str == format!("{}.slack.com", profile_team_domain) {
        info!("Domain matches Slack profile");

        // Slack commands:
        //   channel
        //   file
        //   team

        // Slack query params:
        //      id: ([CDG][A-Z0-9]{8,})
        //    team: (T[A-Z0-9]{8,})
        // message: ([0-9]+\.[0-9]+)

        // Team host:
        //    File:            https://<team-domain-name>.slack.com/messages/<ignored_id>/files/<file_id>
        //    Channel:         https://<team-domain-name>.slack.com/archives/<channel_id>
        //    Channel message: https://<team-domain-name>.slack.com/archives/<channel_id>/p<timestamp_without_decimal>
        //    User:            https://<team-domain-name>.slack.com/team/<user_id>
        //    User?:           https://<team-domain-name>.slack.com/messages/<ignored_id>/team/<user_id>

        if url_path_segments_maybe.is_some() {
            let segments = url_path_segments_maybe.unwrap();
            let resource_type_maybe = segments.get(0);
            let resource_id_maybe = segments.get(1);

            let uri_maybe =
                resource_type_maybe
                    .zip(resource_id_maybe)
                    .map(|(resource_type, resource_id)| {
                        let subresource_id_maybe = segments.get(2);

                        match (resource_type.as_str(), subresource_id_maybe) {
                            ("team", _) => {
                                // User; resource_id: user id
                                // From: https://<team-domain>.slack.com/team/<resource_id>
                                //   To: slack://team?team=<team-id>&id=<resource_id>
                                format!("slack://team?team={}&id={}", profile_team_id, resource_id)
                            }
                            ("files", Some(file_id)) => {
                                // File; resource_id: user id; subresource_id: file id
                                // From https://<team-domain>.slack.com/files/<resource_id>/<file-id>/<filename>
                                //   To slack://file?team=<team-id>&id=<file-id>
                                format!("slack://file?team={}&id={}", resource_id, file_id)
                            }
                            ("archives", Some(message_id)) => {
                                let query_pairs = url.query_pairs();
                                let thread_ts_maybe: Option<String> = query_pairs
                                    .into_iter()
                                    .find(|(key, value)| key == "thread_ts")
                                    .map(|(key, value)| value.to_string());

                                match thread_ts_maybe {
                                    // Channel thread message; resource_id: channel id; subresource_id: message id
                                    // From https://<team-domain>.slack.com/archives/C05BH52KSC8/p1686336166083089?thread_ts=1686336161.925399&cid=C05BH52KSC8
                                    //   To slack://channel?team=<team-id>&id=<resource_id>&message=1686336166.083089&thread_ts=1686232159.321829
                                    Some(thread_ts) => format!(
                                        "slack://channel?team={}&id={}&message={}&thread_ts={}",
                                        profile_team_id, resource_id, message_id, thread_ts
                                    ),
                                    // Channel Message; resource_id: channel id; subresource_id: message id
                                    // From https://<team-domain>.slack.com/archives/<resource_id>/p1647522989096739
                                    //   To slack://channel?team=<team-id>&id=<resource_id>&message=1647522989.096739
                                    None => format!(
                                        "slack://channel?team={}&id={}&message={}",
                                        profile_team_id, resource_id, message_id
                                    ),
                                }
                            }
                            ("archives", None) => {
                                // Channel; resource_id: channel id
                                // From https://<team-domain>.slack.com/archives/<channel_id>
                                //   To slack://channel?team=<team-id>&id=<channel_id>
                                format!(
                                    "slack://channel?team={}&id={}",
                                    profile_team_id, resource_id
                                )
                            }
                            _ => unknown.clone(),
                        }
                    });

            let slack_protocol_url = uri_maybe.unwrap_or(unknown.clone());
            return slack_protocol_url;
        }

        format!("slack://channel?team={}", profile_team_id)
    } else if url_host_str == format!("{}.slack-gov.com", profile_team_domain) {
        // https://mycompany.slack-gov.com/...
        format!("slack://channel?team={}", profile_team_id)
    } else if url_host_str == format!("{}.enterprise.slack.com", profile_team_domain) {
        // https://mycompany.enterprise.slack.com/...
        format!("slack://channel?team={}", profile_team_id)
    } else if url_host_str == "app.slack.com" {
        // https://app.slack.com/client/...
        format!("slack://channel?team={}", profile_team_id)
    } else {
        format!("slack://channel?team={}", profile_team_id)
    };
}
