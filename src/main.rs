mod bit_flags;
mod cli;
mod prelude;
mod settings;

use cli::*;
use prelude::*;
use settings::*;

fn main() {
    let mut cli = Cli::init();
    let settings: Settings = cli.parse_settings();
    let language = cli.language();
    log::info!("generating password in {:?} with {}", language, settings);

    let pass_generator = Xkpasswd::for_language(language);
    
    let result = match settings.padding_strategy {
        PaddingStrategy::AdaptiveFit(len) => {
            pass_generator.gen_pass_fit(&settings, len)
                .unwrap_or_else(|err| {
                    log::error!("AdaptiveFit error: {}", err);
                    std::process::exit(1);
                })
        }
        _ => pass_generator.gen_pass(&settings)
    };
    
    let (passwd, entropy) = result;
    log::info!("calculated entropy: {}", entropy);

    println!("{}", passwd);
}
