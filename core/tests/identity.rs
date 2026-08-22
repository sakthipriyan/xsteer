use xsteer_core::model::{AccountKey, AccountKind};

fn key(institution: &str, masked: &str) -> AccountKey {
    AccountKey {
        institution: institution.to_string(),
        kind: AccountKind::Savings,
        masked_number: masked.to_string(),
    }
}

#[test]
fn mask_formatting_does_not_change_identity() {
    // The same HDFC account, masked three ways across three statement formats.
    let a = key("HDFC Bank", "XXXXXX501234").account_id();
    let b = key("HDFC Bank", "****501234").account_id();
    let c = key("HDFC Bank", "50 1234").account_id();
    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn institution_casing_does_not_change_identity() {
    assert_eq!(
        key("HDFC Bank", "XXXX1234").account_id(),
        key("hdfc bank ", "XXXX1234").account_id()
    );
}

#[test]
fn different_accounts_stay_distinct() {
    assert_ne!(
        key("HDFC Bank", "XXXX1234").account_id(),
        key("HDFC Bank", "XXXX5678").account_id()
    );
    assert_ne!(
        key("HDFC Bank", "XXXX1234").account_id(),
        key("ICICI Bank", "XXXX1234").account_id()
    );
}

#[test]
fn account_kind_separates_a_card_from_a_bank_account() {
    let bank = key("HDFC Bank", "XXXX1234").account_id();
    let card = AccountKey {
        institution: "HDFC Bank".to_string(),
        kind: AccountKind::CreditCard,
        masked_number: "XXXX1234".to_string(),
    }
    .account_id();
    assert_ne!(bank, card);
}
