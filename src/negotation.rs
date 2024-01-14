pub struct NegotiationItem {
    name: String,
    q: f32,
}
pub struct Negotiation {
    params: Vec<NegotiationItem>,
}
impl Negotiation {
    pub fn best<'a>(&self, available: &[&'a str]) -> Option<&String> {
        let mut best = None;
        let mut best_q = 0.0;
        for item in &self.params {
            if item.q > best_q {
                if available.contains(&item.name.as_str()) {
                    best = Some(&item.name);
                    best_q = item.q;
                }
            }
        }
        best
    }

    pub fn parse(accept: String) -> Self {
        accept.split(",").map(|item| {
            let mut iter = item.split(";");
            let name = iter.next().unwrap_or("identity").to_string();
            let q = iter.next().and_then(|q| {
                let mut iter = q.split("=");
                iter.next().and_then(|q| {
                    q.parse::<f32>().ok()
                })
            }).unwrap_or(1.0);
            NegotiationItem {
                name,
                q,
            }
        }).collect()
    }
}
impl FromIterator<NegotiationItem> for Negotiation {
    fn from_iter<T: IntoIterator<Item=NegotiationItem>>(iter: T) -> Self {
        let params = iter.into_iter().collect();
        Negotiation {
            params,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let negotiation = Negotiation::parse("gzip;q=1.0, identity; q=0.5, *;q=0".to_string());
        assert_eq!(negotiation.params.len(), 3);
        assert_eq!(negotiation.params[0].name, "gzip");
        assert_eq!(negotiation.params[0].q, 1.0);
        assert_eq!(negotiation.params[1].name, "identity");
        assert_eq!(negotiation.params[1].q, 0.5);
        assert_eq!(negotiation.params[2].name, "*");
        assert_eq!(negotiation.params[2].q, 0.0);
    }
}