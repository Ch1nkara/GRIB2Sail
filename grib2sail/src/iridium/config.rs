use crate::core::GribError;

use regex::Regex;
use reqwest::{
    Client, Request,
    header::{
        ACCEPT_ENCODING, CONNECTION, CONTENT_LANGUAGE, CONTENT_TYPE, HOST,
        HeaderMap, HeaderValue,
    },
};
use std::net::SocketAddr;

pub enum UrlType {
    GetStatus,
    PerformTask,
}

const BODY_STATUS_START: &str = r#"
<soapenv:Envelope 
  xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/" 
  xmlns:sdk="http://192.168.0.1/sdk/">
   <soapenv:Header/>
   <soapenv:Body>
      <sdk:getStatus>
         <userCredentials>
            <userName>guest</userName>
            <password>guest</password>
         </userCredentials>
         <request>
            <options>
"#;
const BODY_STATUS_END: &str = r#"
               <value></value>
               <dataType>null</dataType>
            </options>
         </request>
      </sdk:getStatus>
   </soapenv:Body>
</soapenv:Envelope>
"#;

const BODY_TASK_1: &str = r#"
<soapenv:Envelope 
  xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/" 
  xmlns:sdk="http://192.168.0.1/sdk/">
   <soapenv:Header/>
   <soapenv:Body>
<sdk:performTask>
         <userCredentials>
            <userName>guest</userName>
            <password>guest</password>
         </userCredentials>
         <taskList>
            <requestList>
               <taskID>2</taskID>
               <options>
                  <name>set state</name>
"#;
const BODY_TASK_2: &str = r#"
                  <dataType>boolean</dataType>
               </options>
               <options>
                   <name>Firewall allow all traffic</name>
                   <value>false</value>
                   <dataType>boolean</dataType>
               </options>
               <options>
                   <name>Firewall exceptions</name>
"#;
const BODY_TASK_3: &str = r#"
                   <dataType>string</dataType>
               </options>
               <options>
                   <name>Enable DNS forwarding</name>
                   <value>false</value>
                   <dataType>boolean</dataType>
               </options>
            </requestList>
         </taskList>
      </sdk:performTask>   </soapenv:Body>
</soapenv:Envelope>
"#;

pub fn get_req(
    client: &Client,
    url_type: UrlType,
    options: Option<(&[(&str, SocketAddr)], usize)>,
) -> Result<Request, GribError> {
    let url = "http://192.168.0.1/sdk/sdk.php";

    // Build headers
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/xml"));
    headers.insert(CONTENT_LANGUAGE, HeaderValue::from_static("en-US"));
    headers.insert(HOST, HeaderValue::from_static("192.168.0.1"));
    headers.insert(CONNECTION, HeaderValue::from_static("Keep-Alive"));
    headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip"));

    let param = match (&url_type, options) {
        (UrlType::GetStatus, None) => {
            headers.insert(
                "SOAPAction",
                HeaderValue::from_static("http://192.168.0.1/sdk/getStatus"),
            );
            "<name>all</name>"
        }
        (UrlType::PerformTask, Some((vs, _))) => {
            headers.insert(
                "SOAPAction",
                HeaderValue::from_static("http://192.168.0.1/sdk/performTask"),
            );
            let formatted = vs
                .iter()
                .map(|(_, addr)| format!("{}-{}-all", addr.ip(), addr.port()))
                .collect::<Vec<String>>()
                .join(";");
            &format!("<value>{}</value>", formatted)
        }
        (_, _) => {
            let mut msg = String::from("Unexpected get_req parameter");
            msg.push_str(" combinaison");
            return Err(GribError::Generic(msg));
        }
    };
    let body = match (url_type, options) {
        (UrlType::GetStatus, None) => {
            format!("{}{}{}", BODY_STATUS_START, param, BODY_STATUS_END)
        }
        (UrlType::PerformTask, Some((_, a))) => {
            let action = format!("<value>{}</value>", a);
            format!(
                "{}{}{}{}{}",
                BODY_TASK_1, action, BODY_TASK_2, param, BODY_TASK_3
            )
        }
        (_, _) => {
            let mut msg = String::from("Unexpected get_req parameter");
            msg.push_str(" combinaison");
            return Err(GribError::Generic(msg));
        }
    };

    Ok(client.post(url).headers(headers).body(body).build()?)
}

pub fn parse_response(
    body: String,
    url: UrlType,
) -> Result<Vec<String>, GribError> {
    let mut regex_vec: Vec<Regex> = Vec::new();

    match url {
        UrlType::GetStatus => {
            regex_vec.push(Regex::new(
                r"Internet connection status</name><value>(\d+)",
            )?);
            regex_vec.push(Regex::new(
                r"Iridium signal strength</name><value>(\d+)",
            )?);
        }
        _ => {
            regex_vec.push(Regex::new(r"\bA2:\s*(\w+)\b")?);
        }
    }

    let mut results: Vec<String> = Vec::new();

    for regex in regex_vec.iter() {
        for cap in regex.captures_iter(&body) {
            if let Some(m) = cap.get(1) {
                results.push(m.as_str().to_string());
            } else {
                let msg = String::from("Failed to parse iridium response");
                return Err(GribError::Generic(msg));
            }
        }
    }
    Ok(results)
}
