#![no_std]
use core::ops::Add;

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
    ContadorPorUsuario(Address),
    LimiteCaracteres,
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
    pub fn set_limite(env: Env, caller: Address, limite: u32) -> Result<(), Error> {
        // Verificar que el contrato esté inicializado
        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::NoInicializado);
        }

        // Verificar que el caller sea el admin
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NoInicializado)?;

        if caller != admin {
            return Err(Error::NoAutorizado);
        }

        // Guardar el límite
        env.storage()
            .instance()
            .set(&DataKey::LimiteCaracteres, &limite);
        env.storage().instance().extend_ttl(17280, 17280);

        Ok(())
    }

    pub fn hello(env: Env, usuario: Address, nombre: String) -> Result<Symbol, Error> {
        if nombre.len() == 0 {
            return Err(Error::NombreVacio);
        }
        let limite: u32 = env
            .storage()
            .instance()
            .get(&DataKey::LimiteCaracteres)
            .unwrap_or(32);
        if nombre.len() > limite as u32 {
            return Err(Error::NombreMuyLargo);
        }
        // Incrementamos contador global
        let key_contador = DataKey::ContadorSaludos;
        let contador: u32 = env.storage().instance().get(&key_contador).unwrap_or(0);
        env.storage().instance().set(&key_contador, &(contador + 1));

        // Guardamos el último saludo: construimos la clave una sola vez clonando usuario
        let ultimo_key = DataKey::UltimoSaludo(usuario.clone());
        env.storage().persistent().set(&ultimo_key, &nombre);
        env.storage()
            .persistent()
            .extend_ttl(&ultimo_key, 17280, 17280);

        // Extendemos TTL del storage de instancia si lo necesitamos
        env.storage().instance().extend_ttl(17280, 17280);

        // Contador por usuario: usamos otra clave (nuevamente clonamos usuario para crear la clave)
        let key_contador_usuario = DataKey::ContadorPorUsuario(usuario.clone());
        let contador_usuario: u32 = env
            .storage()
            .instance()
            .get(&key_contador_usuario)
            .unwrap_or(0);

        let nuevo_contador = contador_usuario + 1;
        env.storage()
            .instance()
            .set(&key_contador_usuario, &nuevo_contador);

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

    pub fn get_contador_usuario(env: Env, usuario: Address) -> u32 {
        let key = DataKey::ContadorPorUsuario(usuario);
        env.storage().instance().get(&key).unwrap_or(0)
    }
    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Admin no inicializado")
    }

    pub fn transfer_admin(env: Env, caller: Address, nuevo_admin: Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NoInicializado)?;
        if caller != admin {
            return Err(Error::NoAutorizado);
        }

        env.storage().instance().set(&DataKey::Admin, &nuevo_admin);

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

    #[test]
    fn test_contador_por_usuario() {
        let env = Env::default();
        let contract_id = env.register_contract(None, HelloContract);
        let client = HelloContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin);

        let usuario = Address::generate(&env);
        let nombre = String::from_str(&env, "Tiburona");

        // Al principio debe ser 0
        assert_eq!(client.get_contador_usuario(&usuario), 0);

        // Después de un saludo, debe ser 1
        client.hello(&usuario, &nombre);
        assert_eq!(client.get_contador_usuario(&usuario), 1);

        // Si saluda otra vez, debe ser 2
        client.hello(&usuario, &nombre);
        assert_eq!(client.get_contador_usuario(&usuario), 2);
    }
    #[test]
    fn test_transfer_admin_exitoso() {
        let env = Env::default();
        let contract_id = env.register_contract(None, HelloContract);
        let client = HelloContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin);

        let nuevo_admin = Address::generate(&env);

        // Admin actual transfiere correctamente
        client.transfer_admin(&admin, &nuevo_admin);

        // Verificamos que el admin haya cambiado
        let admin_actual = client.get_admin();
        assert_eq!(admin_actual, nuevo_admin);

        // El nuevo admin puede transferir a otro
        let otro_admin = Address::generate(&env);
        client.transfer_admin(&nuevo_admin, &otro_admin);
        assert_eq!(client.get_admin(), otro_admin);
    }
}
