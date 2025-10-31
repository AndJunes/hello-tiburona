#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, Env, String, Symbol,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    NombreVacio = 1,
    NombreMuyLargo = 2,
    NoAutorizado = 3,
    NoInicializado = 4,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    ContadorSaludos,
    UltimoSaludo(Address),
}

#[contract]
pub struct HelloContract;

#[contractimpl]
impl HelloContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        // Verifica si ya existe un admin
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::NoInicializado);
        }

        // Almacena la dirección del administrador
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::ContadorSaludos, &0u32);
        env.storage().instance().extend_ttl(17280, 17280);

        // Retorna éxito con el valor unitario
        Ok(())
    }

    pub fn hello(env: Env, usuario: Address, nombre: String) -> Result<Symbol, Error> {
        if nombre.len() == 0 {
            return Err(Error::NombreVacio);
        }
        if nombre.len() > 32 {
            return Err(Error::NombreMuyLargo);
        }
        //Incrementamos el contador de saludos
        let key_contador = DataKey::ContadorSaludos;
        let contador: u32 = env.storage().instance().get(&key_contador).unwrap_or(0);
        env.storage().instance().set(&key_contador, &(contador + 1));
        env.storage()
            .persistent()
            .set(&DataKey::UltimoSaludo(usuario.clone()), &nombre);

        env.storage()
            .persistent()
            .extend_ttl(&DataKey::UltimoSaludo(usuario), 17280, 17280);

        env.storage().instance().extend_ttl(17280, 17280);

        Ok(Symbol::new(&env, "Hola"))
    }

    pub fn get_contador(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::ContadorSaludos)
            .unwrap_or(0)
    }

    pub fn get_ultimo_saludo(env: Env, usuario: Address) -> Option<String> {
        env.storage()
            .persistent()
            .get(&DataKey::UltimoSaludo(usuario))
    }

    pub fn reset_contador(env: Env, caller: Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NoInicializado)?;
        if caller != admin {
            return Err(Error::NoAutorizado);
        }
        env.storage()
            .instance()
            .set(&DataKey::ContadorSaludos, &0u32);
        env.storage().instance().extend_ttl(17280, 17280);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env};

    #[test]
    fn test_initialize() {
        let env = Env::default();
        let contract_id = env.register_contract(None, HelloContract);
        let client = HelloContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin);

        assert_eq!(client.get_contador(), 0);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1)")]
    fn test_nombre_vacio() {
        let env = Env::default();
        let contract_id = env.register_contract(None, HelloContract);
        let client = HelloContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin);

        let usuario = Address::generate(&env);
        client.hello(&usuario, &String::from_str(&env, ""));
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #4)")]
    fn test_no_reinicializar() {
        let env = Env::default();
        let contract_id = env.register_contract(None, HelloContract);
        let client = HelloContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin);

        // Intentar inicializar de nuevo debería fallar
        client.initialize(&admin);
    }

    #[test]
    fn test_hello_exitoso() {
        let env = Env::default();
        let contract_id = env.register_contract(None, HelloContract);
        let client = HelloContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin);

        let usuario = Address::generate(&env);
        let nombre = String::from_str(&env, "Tiburón");

        // Primer saludo
        client.hello(&usuario, &nombre);
        assert_eq!(client.get_contador(), 1);
        assert_eq!(client.get_ultimo_saludo(&usuario), Some(nombre.clone()));

        // Segundo saludo
        client.hello(&usuario, &nombre);
        assert_eq!(client.get_contador(), 2);
    }

    #[test]
    fn test_reset_solo_admin() {
        let env = Env::default();
        let contract_id = env.register_contract(None, HelloContract);
        let client = HelloContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin);

        let usuario = Address::generate(&env);
        let nombre = String::from_str(&env, "Tiburón");

        client.hello(&usuario, &nombre);
        assert_eq!(client.get_contador(), 1);

        // El admin puede resetear el contador
        client.reset_contador(&admin);
        assert_eq!(client.get_contador(), 0);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #3)")]
    fn test_reset_no_autorizado() {
        let env = Env::default();
        let contract_id = env.register_contract(None, HelloContract);
        let client = HelloContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin);

        let usuario = Address::generate(&env);
        // Un usuario que no es admin no debería poder resetear el contador
        client.reset_contador(&usuario);
    }
}
