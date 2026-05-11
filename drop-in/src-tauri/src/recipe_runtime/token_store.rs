use super::{secret_redaction_registry, status::RecipeSecretInput};

const SERVICE_NAME: &str = "OpenNest Recipe Secrets";

pub fn save(app_id: &str, secrets: Vec<RecipeSecretInput>) -> Result<(), String> {
    for secret in secrets {
        secret_redaction_registry::register_secret(&secret.value);
        let entry = keyring::Entry::new(SERVICE_NAME, &format!("{}:{}", app_id, secret.id)).map_err(|e| e.to_string())?;
        entry.set_password(&secret.value).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn get(app_id: &str, secret_id: &str) -> Option<String> {
    let entry = keyring::Entry::new(SERVICE_NAME, &format!("{}:{}", app_id, secret_id)).ok()?;
    let value = entry.get_password().ok()?;
    secret_redaction_registry::register_secret(&value);
    Some(value)
}


pub fn delete(app_id: &str, secret_id: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(SERVICE_NAME, &format!("{}:{}", app_id, secret_id)).map_err(|e| e.to_string())?;
    match entry.delete_credential() {
        Ok(_) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

pub fn delete_many(app_id: &str, secret_ids: &[String]) -> Result<(), String> {
    let mut failures = Vec::new();
    for secret_id in secret_ids {
        if let Err(error) = delete(app_id, secret_id) {
            failures.push(format!("{}: {}", secret_id, error));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!("failed to delete some secrets: {}", failures.join("; ")))
    }
}
