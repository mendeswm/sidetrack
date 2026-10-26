# Dependências de desenvolvimento — Ubuntu 26.04 LTS

Este documento lista a base recomendada para compilar e testar o fork do Sidetrack. A lista exata deve ser ajustada depois da auditoria dos arquivos Gradle e do código nativo. Não instale versões aleatórias de Gradle/AGP: use o Gradle Wrapper do repositório e respeite a compatibilidade exigida pelo projeto.

## 1. Pacotes do Ubuntu

Pacotes base para ferramentas de compilação, Git, utilitários Android e dependências nativas comuns:

```bash
sudo apt update
sudo apt install \
  git git-lfs curl wget unzip zip xz-utils file \
  build-essential pkg-config cmake ninja-build clang lld \
  openjdk-17-jdk \
  adb
```

Observações:
- Se o projeto/Android Gradle Plugin exigir outro JDK, siga a versão exigida pelo wrapper e pelos plugins. O upstream documenta Java 17.
- `cmake`, `ninja-build`, `clang` e `lld` são ferramentas úteis para dependências nativas; nem todas serão necessariamente chamadas pelo build atual.
- Instale `libssl-dev` ou bibliotecas `*-dev` adicionais apenas se o erro de compilação ou a documentação das dependências nativas demonstrar necessidade.
- `git-lfs` é opcional e só necessário se o repositório utilizar Git LFS.

## 2. Rust

Recomendação para desenvolvimento geral: instalar Rust pelo `rustup`, que permite controlar a versão e os targets. No Ubuntu, evite misturar sem necessidade o `rustc`/`cargo` do APT com toolchains de `rustup`.

Instale `rustup` pelo método oficial documentado em https://rustup.rs/ e depois configure a toolchain estável e os targets exigidos pelo aparelho.

Targets Android prováveis (confirme a ABI real do Oilsky M308 antes de decidir quais são necessários):

```bash
rustup toolchain install stable
rustup default stable
rustup component add rustfmt clippy
rustup target add aarch64-linux-android armv7-linux-androideabi
```

Targets:
- `aarch64-linux-android`: Android ARM64 (`arm64-v8a`).
- `armv7-linux-androideabi`: Android ARM de 32 bits (`armeabi-v7a`).

Não presuma que o aparelho usa uma ABI específica. Confira o dispositivo via ADB quando estiver conectado. Instale somente os targets necessários se quiser reduzir o ambiente.

## 3. Android SDK e NDK

Instale Android SDK Command-line Tools, Platform Tools, Build Tools e NDK a partir das ferramentas oficiais do Android. É possível usar Android Studio ou o `sdkmanager`.

O upstream Sidetrack documenta Android SDK API 34 e NDK `27.0.12077973`; isso é uma referência inicial, não garantia de que essa combinação seja a correta para o checkout atual. Confirme a versão no projeto e mantenha-a fixada/reprodutível. A versão do NDK indicada pelo upstream pode ser instalada lado a lado com outras versões.

Componentes necessários em geral:
- Android SDK Platform correspondente ao `compileSdk` do projeto.
- Android SDK Build-Tools exigido pelo Android Gradle Plugin.
- Android SDK Platform-Tools (`adb`).
- Android NDK compatível com o build Rust/JNI.
- CMake, se solicitado pela configuração do projeto ou por uma dependência nativa.

Configure `ANDROID_SDK_ROOT` (ou `ANDROID_HOME`, conforme as ferramentas utilizadas) para a localização do SDK e `ANDROID_NDK_HOME` para a versão efetivamente instalada. Evite colocar caminhos pessoais absolutos no repositório.

## 4. cargo-ndk

O upstream usa `cargo-ndk` para compilar a biblioteca Rust para Android. Instale-o com Cargo:

```bash
cargo install cargo-ndk
```

Verifique a documentação e o `build.gradle.kts` do checkout antes de alterar tarefas Gradle ou argumentos de build.

## 5. Gradle e Kotlin

- Use `./gradlew` e `gradle/wrapper/gradle-wrapper.properties` fornecidos pelo repositório.
- Não é necessário instalar Gradle globalmente.
- A versão do Android Gradle Plugin, Kotlin, Compose e JDK deve permanecer coerente com os arquivos de build.
- Não atualize todas as dependências em lote como tentativa de resolver incompatibilidade com Android 9. Identifique primeiro a API/dependência bloqueadora.

## 6. Ferramentas úteis, opcionais

```bash
sudo apt install \
  gdb lldb jq ripgrep
```

- `gdb`/`lldb`: diagnóstico de falhas nativas, quando aplicável.
- `jq`: inspeção de JSON em logs e scripts.
- `ripgrep` (`rg`): busca rápida no código.
- Android Studio: opcional, útil para inspeção de layouts, Logcat, profiling e gerenciamento do SDK. Não é necessário se o desenvolvimento for feito pelo terminal e Kilo Code.

## 7. Verificação do ambiente

Após instalar e configurar o ambiente, verifique cada ferramenta individualmente e compare os resultados com as versões exigidas pelo repositório. Não considere o ambiente validado até que o build debug termine sem erros.

Para confirmar ABI e versão Android do dispositivo, conecte o player por USB, habilite ADB se o firmware permitir e consulte as propriedades do dispositivo com `adb`. Não inclua a saída de propriedades potencialmente identificadoras em relatórios públicos sem necessidade.

## 8. Referências oficiais

- Sidetrack e requisitos/build upstream: https://github.com/qfertig/sidetrack
- Rust no Ubuntu: https://ubuntu.com/developers/docs/howto/rust-setup/
- Targets Rust para Android: https://doc.rust-lang.org/rustc/platform-support/android.html
- Instalação do Android NDK: https://developer.android.com/studio/projects/install-ndk
- Instalação oficial do Rust: https://rustup.rs/

## Nota sobre Ubuntu 26.04

Ubuntu 26.04 fornece toolchains recentes, mas isso não significa que a versão mais nova de cada ferramenta seja a mais compatível com um projeto Android existente. Priorize as versões exigidas pelo Gradle Wrapper, pelo Android Gradle Plugin e pela configuração do NDK no repositório.
