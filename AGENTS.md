# AGENTS.md — Diretrizes para o fork do Sidetrack no Oilsky M308

## 1. Missão

Adaptar este repositório (fork de `qfertig/sidetrack`) para criar um cliente Spotify não oficial, leve e utilizável no Oilsky M308, com Android 9 (API 28), tela touch de 480 × 640, CPU ARM ainda a confirmar e 2 GB de RAM.

O objetivo primário é obter reprodução online estável, com a melhor qualidade de streaming que o Spotify e a versão integrada do `librespot` disponibilizarem. Não implementar reprodução offline como requisito inicial.

## 2. Regras de trabalho obrigatórias

1. **Inspecione antes de alterar.** Leia `README.md`, `DESIGN.md`, `settings.gradle.kts`, os arquivos Gradle, o código em `app/`, `native/`, submódulos e scripts de build. Confirme o estado real do repositório; não presuma que a documentação descreve perfeitamente o código atual.
2. **Faça mudanças incrementais e verificáveis.** Primeiro apresente um diagnóstico breve: arquitetura atual, `minSdk`, `targetSdk`, versões de Gradle/AGP/Kotlin/Java, dependências incompatíveis com API 28, ABI nativa e caminho de áudio. Depois proponha etapas pequenas e execute testes após cada mudança relevante.
3. Preserve o funcionamento existente sempre que possível. Não reescreva a aplicação inteira nem substitua `librespot` sem evidência técnica e justificativa.
4. Não faça alterações destrutivas, remova funcionalidades ou troque a arquitetura sem explicar a razão e o impacto.
5. Não invente resultados de build, testes ou execução em hardware. Distinga claramente o que foi verificado no emulador, em dispositivo real ou apenas por inspeção.
6. Antes de concluir, informe os arquivos alterados, as decisões, os testes executados, os resultados e as limitações ainda não verificadas.
7. Mantenha o idioma da documentação e das mensagens da interface consistente. A interface pode começar em português brasileiro; evite textos longos e rótulos que não caibam na tela.

## 3. Primeira tarefa: auditoria do projeto

Antes de modificar o código:

- Leia o `README.md` e `DESIGN.md` e examine a árvore do repositório.
- Inspecione `.gitmodules` e confirme se o submódulo do `librespot` está inicializado e qual commit está sendo usado.
- Identifique `minSdk`, `targetSdk`, `compileSdk`, versões de Android Gradle Plugin, Gradle Wrapper, Kotlin, Java e Jetpack Compose.
- Localize o código de ponte JNI, a implementação Rust, o mecanismo de saída de áudio e o serviço de reprodução.
- Identifique chamadas Android introduzidas após API 28 e dependências que exijam API superior.
- Confirme quais ABIs o projeto compila e quais o Oilsky M308 realmente utiliza. Não deduza a ABI apenas pelo número de núcleos ou pela descrição comercial.
- Verifique como são armazenados tokens, credenciais e dados de sessão; não registre segredos em logs.
- Registre um baseline de tamanho do APK, tempo de inicialização, uso aproximado de memória e consumo de CPU quando for possível medir.

Entregue primeiro um resumo de diagnóstico e um plano priorizado. Não comece com uma migração ampla sem entender a arquitetura atual.

## 4. Compatibilidade Android 9

- Android 9 corresponde à API 28. A aplicação deve instalar e executar nessa versão.
- Configure `minSdk` para 28 somente depois de verificar e corrigir incompatibilidades reais. `targetSdk` e `compileSdk` têm funções diferentes de `minSdk`; não os trate como equivalentes.
- Prefira APIs disponíveis na API 28 ou forneça alternativas compatíveis para APIs mais recentes.
- Verifique compatibilidade das versões de AndroidX, Compose, bibliotecas de imagens, serviços em primeiro plano e notificações. Faça downgrade seletivo apenas quando necessário e compatível.
- Confirme permissões e comportamento de serviços de reprodução em segundo plano no Android 9. Não introduza dependências de Google Play Services; o projeto original se propõe a funcionar sem eles.
- Teste instalação, inicialização, login/pareamento, reprodução, pausa, troca de faixa, controle de volume, interrupções de áudio e retorno do aplicativo.
- Não prometa compatibilidade apenas porque o APK compila. O teste no Oilsky M308 é o critério final.

## 5. Interface para tela 480 × 640

Priorize uma interface touch simples, compacta e responsiva:

- Projete primeiro para 480 × 640 pixels em orientação retrato, mas respeite densidade real, insets e tamanho lógico em dp.
- Evite layouts que dependam de tela grande, menus extensos, painéis simultâneos ou rolagem horizontal.
- Use controles grandes, com espaçamento suficiente para toque e estados visuais claros.
- A tela de reprodução deve dar prioridade a capa, título, artista, progresso, anterior, reproduzir/pausar e próxima.
- Coloque fila, busca, playlists e configurações em telas separadas e simples.
- Reduza animações, sombras, recomposições desnecessárias e atualizações frequentes da interface.
- Carregue imagens sob demanda, dimensione-as antes de exibir e limite o cache em memória.
- Evite manter listas completas e objetos de imagem na memória quando não forem necessários.
- Mantenha acessibilidade básica: contraste legível, descrições de controles e tamanho de toque adequado.
- Não remova a compatibilidade com controles físicos existentes se isso for barato de preservar, mas não os torne requisito: o M308 é touch.

## 6. Áudio, streaming e desempenho

- Preserve `librespot` como núcleo de sessão e streaming, salvo incompatibilidade demonstrada.
- Determine e documente o caminho de áudio atual entre Rust/JNI e Android `AudioTrack`; procure cópias excessivas de buffers, alocações frequentes e bloqueios na thread de interface.
- Evite enviar grandes blocos de PCM pela JNI se houver uma alternativa segura e comprovadamente melhor; qualquer mudança deve ser medida e testada.
- Garanta que decodificação e operações de rede não bloqueiem a UI.
- Use buffers de áudio razoáveis e tratamento de backpressure; evite tanto underruns quanto consumo de memória desnecessário.
- Implemente estados explícitos para conectando, carregando, reproduzindo, pausado, erro de rede e sessão expirada.
- A meta de qualidade é a maior opção efetivamente oferecida pela integração e pela conta. A documentação atual do Sidetrack informa até 320 kbps Ogg Vorbis; não chame isso de lossless.
- Não implemente extração de conteúdo, contorno de DRM, evasão de controles de acesso ou mecanismos para burlar limitações do Spotify.
- Streaming online é suficiente para a primeira versão. Não acrescente download/offline ou cache persistente de áudio como parte desta tarefa.

## 7. Segurança e conta

- Não grave senhas, tokens, cookies, chaves ou payloads de autenticação nos logs.
- Use armazenamento privado do aplicativo para dados de sessão e siga as práticas de segurança existentes, melhorando-as quando necessário.
- Não inclua segredos, keystores, senhas de assinatura ou arquivos locais de configuração no Git.
- Preserve avisos de que se trata de um cliente de terceiros não oficial. O uso de clientes não oficiais pode envolver risco para a conta e está sujeito aos termos e às mudanças do serviço.
- Não afirme que o projeto é aprovado, certificado ou endossado pelo Spotify.

## 8. Build e qualidade do código

- Respeite o Gradle Wrapper do repositório e a matriz de compatibilidade entre Gradle, Android Gradle Plugin, Kotlin e JDK.
- Use Rust estável, exceto se o projeto demonstrar necessidade justificada de outra versão.
- Preserve as versões de dependências no início; atualize somente quando necessário para API 28, segurança ou build reproduzível.
- Mantenha as alterações pequenas e explique qualquer mudança em versões ou dependências.
- Execute, quando disponíveis e pertinentes: testes Rust, testes unitários Android, lint, build debug e build release não assinado.
- Verifique os binários nativos dentro do APK e confirme a ABI pretendida.
- Não coloque keystore real no repositório. Se a assinatura exigir configuração local, documente-a sem segredos.
- Evite otimizações prematuras. Meça antes/depois quando ferramentas e dispositivo permitirem.

## 9. Critérios de aceitação

A tarefa não está concluída apenas com uma compilação bem-sucedida. Procure satisfazer estes critérios:

1. APK instala e abre em Android 9 / API 28.
2. A interface é utilizável por toque em 480 × 640, sem controles essenciais cortados.
3. O aplicativo conecta-se e inicia streaming online com uma conta elegível.
4. Play/pause, próxima/anterior, seek quando suportado, volume e estado de reprodução funcionam.
5. O áudio é emitido pelo dispositivo sem falhas persistentes, travamentos ou underruns audíveis durante um teste prolongado.
6. A reprodução não bloqueia a interface; alternar telas e retornar ao player não encerra a sessão indevidamente.
7. O consumo de memória e CPU foi medido quando possível e não piorou sem justificativa.
8. O projeto continua compilável por instruções documentadas, sem depender de credenciais ou arquivos secretos locais.
9. Limitações não testadas no Oilsky M308 são declaradas explicitamente.

## 10. Ordem de implementação sugerida

1. Auditoria e baseline, sem mudanças funcionais.
2. Corrigir bloqueadores de compilação e incompatibilidades necessárias para API 28.
3. Compilar para a ABI confirmada do dispositivo e validar JNI/áudio.
4. Ajustar a interface para 480 × 640 e navegação touch.
5. Reduzir uso de memória, imagens, animações e trabalho na thread principal com base em medições.
6. Validar streaming, interrupções, retomada e estabilidade.
7. Atualizar documentação com build, instalação, testes, limitações e procedimento de diagnóstico.

## 11. Escopo negativo

Não redesenhe o protocolo Spotify, não substitua o núcleo de streaming por uma implementação própria, não adicione recursos offline, não migre para uma arquitetura nova sem necessidade e não remova funcionalidades do upstream sem justificar. Priorize compatibilidade, estabilidade e usabilidade no aparelho real.
