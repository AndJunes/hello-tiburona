# Soroban Project

## Project Structure

This repository uses the recommended structure for a Soroban project:

```text
.
├── contracts
│   └── hello_world
│       ├── src
│       │   ├── lib.rs
│       │   └── test.rs
│       └── Cargo.toml
├── Cargo.toml
└── README.md
```

- New Soroban contracts can be put in `contracts`, each in their own directory. There is already a `hello_world` contract in there to get you started.
- If you initialized this project with any other example contracts via `--with-example`, those contracts will be in the `contracts` directory as well.
- Contracts should have their own `Cargo.toml` files that rely on the top-level `Cargo.toml` workspace for their dependencies.
- Frontend libraries can be added to the top-level directory as well. If you initialized this project with a frontend template via `--frontend-template` you will have those files already included.

---

## Explicación en español — ¿Qué construí?

Has creado un contrato inteligente para Soroban (la plataforma de smart contracts de Stellar) que funciona como un "libro de visitas" simple. En palabras coloquiales:

- Permite que cualquier usuario deje un saludo (un texto corto).
- Guarda el último saludo de cada usuario y lleva un contador global de saludos.
- Al crear el contrato se fija un administrador (admin). Solo el admin puede reiniciar el contador.
- Se validan entradas: no se aceptan nombres vacíos ni nombres muy largos (más de 32 caracteres).
- Los datos almacenados tienen un TTL (tiempo de vida) para que no se mantengan indefinidamente.

Es una muestra completa y segura de cómo estructurar un contrato: almacenamiento, permisos básicos y validaciones.

### Funciones principales del contrato

- `initialize(env, admin: Address) -> Result<(), Error>`: inicializa el contrato y guarda la dirección del administrador. Falla si ya hay un admin.
- `hello(env, usuario: Address, nombre: String) -> Result<Symbol, Error>`: permite dejar un saludo; valida `nombre`, incrementa el contador y guarda el último saludo del `usuario`.
- `get_contador(env) -> u32`: devuelve el contador global de saludos.
- `get_ultimo_saludo(env, usuario: Address) -> Option<String>`: devuelve el último saludo de un usuario (si existe).
- `reset_contador(env, caller: Address) -> Result<(), Error>`: reinicia el contador —solo puede hacerlo el admin.

### Errores definidos

- `NombreVacio`: cuando el nombre es cadena vacía.
- `NombreMuyLargo`: cuando el nombre excede 32 caracteres.
- `NoAutorizado`: cuando alguien intenta hacer una acción reservada al admin.
- `NoInicializado`: cuando se espera que el contrato esté inicializado pero no lo está.

### TTL (tiempo de vida)

El contrato usa `extend_ttl` en el almacenamiento. En este proyecto se configuró un TTL amplio (por ejemplo 17.280 ledgers ≈ 1 día) durante pruebas. Ajusta ese valor según necesidad en producción.

## Cómo compilar y probar (comandos útiles)

Comandos que usamos y que conviene tener a mano antes de subir el repo a GitHub:

1. Instalar targets WASM necesarios (si no están):

```powershell
rustup target add wasm32-unknown-unknown
rustup target add wasm32v1-none
```

2. Compilar el contrato a WASM (target directo):

```powershell
cargo build --target wasm32-unknown-unknown --release -p hello-world
```

3. Usar la herramienta `soroban` para construir (genera artefactos para despliegue):

```powershell
soroban contract build
```

4. Ejecutar pruebas unitarias locales:

```powershell
cargo test
```

Notas: si ves advertencias sobre métodos obsoletos (`register_contract` vs `register`) son deprecations de la SDK; el código de pruebas usa la forma compatible para el entorno de pruebas local.

## Qué verificar antes de publicar a GitHub

- Incluye el archivo `Cargo.toml` del contrato y el top-level `Cargo.toml` en el repositorio.
- Mantén el código de tests junto al contrato (ya están en `contracts/hello-world/src/lib.rs`).
- Añade instrucciones en este README para que otros desarrolladores puedan compilar y ejecutar las pruebas localmente.

## Siguientes pasos recomendados

- Documentar en el README cómo desplegar el `.wasm` en una red de pruebas de Stellar usando la CLI de Soroban.
- Añadir ejemplos de invocación (scripts) o un pequeño cliente JS/TS si vas a exponer la funcionalidad desde una web.
- Considerar aumentar/reducir el TTL según la política de retención que necesites.
