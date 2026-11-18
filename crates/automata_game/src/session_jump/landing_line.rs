pub fn landing_company_line(
    number: u32,
    partner_linked: bool,
    partner_watches: bool,
    partner_body_present: bool,
) -> String {
    let link = if partner_linked {
        "linked"
    } else {
        "not linked"
    };
    let body = match (partner_body_present, partner_watches) {
        (true, _) => "present",
        (false, true) => {
            "not made: the partner watches this copy's hero here, by the watching rule"
        }
        (false, false) => "not here yet",
    };
    format!(
        "session jump #{number}: after the landing the partner is {link} and its body is {body}"
    )
}
