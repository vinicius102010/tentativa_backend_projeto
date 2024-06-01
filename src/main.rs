#![allow(dead_code)]
#![allow(unused_imports)]

mod utilitarios;
mod integracao_bd;
mod criptografia;

use mysql::*;
use mysql::prelude::*;
use utilitarios::*;
use integracao_bd::*;


fn main() {
    //integracao_bd::conectar_banco_dados();
    //if integracao_bd::criar_usuario(){
    //   println!("Parabens os dados foram inseridos corretamente");
    //}
    integracao_bd::consulta_bd();
    utilitarios::esperar_enter();
}
