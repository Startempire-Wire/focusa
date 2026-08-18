// Spec 173 C1 — focusa admin grant-license
pub fn grant_license(email: &str, product: &str) -> String { format!("granted {product} to {email}") }
