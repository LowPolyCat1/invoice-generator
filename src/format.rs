use num_format::{Locale, ToFormattedString};

#[derive(Debug, Clone)]
pub enum SymbolPosition {
    Before,
    After,
}

#[derive(Debug, Clone)]
pub struct CurrencyInfo {
    pub symbol: String,
    pub symbol_position: SymbolPosition,
    pub decimal_digits: usize,
}

pub fn get_currency_info(currency_code: &str) -> CurrencyInfo {
    match currency_code {
        "EUR" => CurrencyInfo {
            symbol: "€".to_string(),
            symbol_position: SymbolPosition::After,
            decimal_digits: 2,
        },
        "USD" => CurrencyInfo {
            symbol: "$".to_string(),
            symbol_position: SymbolPosition::Before,
            decimal_digits: 2,
        },
        "JPY" => CurrencyInfo {
            symbol: "¥".to_string(),
            symbol_position: SymbolPosition::Before,
            decimal_digits: 0,
        },
        _ => CurrencyInfo {
            symbol: currency_code.to_string(),
            symbol_position: SymbolPosition::After,
            decimal_digits: 2,
        },
    }
    // TODO: add more currencies, JPY only here for testing 0 decimal digits
}

pub fn format_currency(value: f64, currency_code: &str, locale: &Locale) -> String {
    let info = get_currency_info(currency_code);

    let factor = 10u64.pow(info.decimal_digits as u32) as f64;
    let rounded = (value * factor).round() as u64;

    let units = rounded / (factor as u64);
    let cents = rounded % (factor as u64);

    let formatted_units = units.to_formatted_string(locale);

    let decimal_sep = match locale {
        Locale::de | Locale::fr | Locale::it | Locale::ja => ',',
        _ => '.',
    };

    let number_str = if info.decimal_digits == 0 {
        formatted_units
    } else {
        format!(
            "{}{}{:0width$}",
            formatted_units,
            decimal_sep,
            cents,
            width = info.decimal_digits
        )
    };

    match info.symbol_position {
        SymbolPosition::Before => format!("{}{}", info.symbol, number_str),
        SymbolPosition::After => format!("{} {}", number_str, info.symbol),
    }
}

pub fn get_locale_by_code(code: &str) -> Locale {
    match code {
        "de" | "de-DE" => Locale::de,
        "en" | "en-US" => Locale::en,
        "fr" | "fr-FR" => Locale::fr,
        "it" | "it-IT" => Locale::it,
        "es" | "es-ES" => Locale::es,
        _ => Locale::en,
    }
    // TODO: support more Locale
}
