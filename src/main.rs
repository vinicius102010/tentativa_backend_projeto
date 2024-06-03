#![allow(dead_code)]
#![allow(unused_imports)]

mod criptografia;
mod integracao_bd;
mod utilitarios;

use integracao_bd::BancoDadosConn;
use utilitarios::*;

fn main() {
    let url = "mysql://myadmin:357159Vinic@meuserverdemo-mysql.mysql.database.azure.com:3306/projeto_rust";
    let mut conn_bd = BancoDadosConn::new(url);

    loop {
        utilitarios::limpar_tela();
        println!("Escolha uma opção:");
        println!("1. Criar usuário");
        println!("2. Fazer login");
        println!("3. Sair");

        let mut opcao = String::new();
        std::io::stdin()
            .read_line(&mut opcao)
            .expect("Falha ao ler a opção");
        let opcao = opcao.trim();

        match opcao {
            "1" => {
                if conn_bd.criar_usuario() {
                    println!("Parabéns, os dados foram inseridos corretamente.");
                } else {
                    println!("Erro ao inserir os dados.");
                }
            }
            "2" => {
                if conn_bd.fazer_login() {
                    loop {
                        utilitarios::limpar_tela();
                        println!("Escolha uma opção:");
                        println!("1. Inserir senha");
                        println!("2. Consultar senhas");
                        println!("3. Sair");
                        let mut opcao2 = String::new();
                        std::io::stdin()
                            .read_line(&mut opcao2)
                            .expect("Falha ao ler a opção");
                        let opcao2 = opcao2.trim();

                        match opcao2 {
                            "1" => conn_bd.inserir_senha(),
                            "2" => {
                                conn_bd.consultar_senhas();
                                utilitarios::esperar_enter()
                            }
                            "3" => break,
                            _ => println!("Opção inválida!"),
                        }
                    }
                } else {
                    println!("Falha no login. Usuário ou senha incorretos.");
                }
            }

            "3" => break,
            _ => println!("Opção inválida!"),
        }
    }
}
