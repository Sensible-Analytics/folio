mod anz;
mod beyond;
mod bom;
mod cba;
mod ibkr;
mod ing;

pub fn get_automation_script(bank_key: &str, years_back: u32) -> Option<String> {
    let script = match bank_key {
        "ING" => ing::ING_SCRIPT,
        "CBA" => cba::CBA_SCRIPT,
        "ANZ" => anz::ANZ_SCRIPT,
        "BOM" => bom::BOM_SCRIPT,
        "BEYOND" => beyond::BEYOND_SCRIPT,
        "IBKR" => ibkr::IBKR_SCRIPT,
        _ => return None,
    };
    Some(script.replace("__YEARS_BACK__", &years_back.to_string()))
}
