# CONTEXT.md — Contexto do projeto Oilsky M308

## 1. Visão geral

Este repositório é um fork do projeto Sidetrack, originalmente publicado em:

- Repositório upstream: https://github.com/qfertig/sidetrack
- Núcleo de streaming relacionado: librespot

O objetivo é adaptar o Sidetrack para uso no player portátil Oilsky M308. A aplicação atual não atende ao dispositivo porque o projeto upstream declara uma versão mínima do Android superior à instalada no aparelho, e a interface foi considerada difícil de utilizar em uma tela tão pequena.

O fork deve priorizar compatibilidade, estabilidade, usabilidade e eficiência, preservando a arquitetura e as funcionalidades existentes sempre que isso for tecnicamente viável.

## 2. Dispositivo-alvo

Especificações informadas para o Oilsky M308:

- Sistema operacional: Android 9.0 (API 28).
- Processador: 8 núcleos, 1,5 GHz.
- Memória RAM: 2 GB.
- Armazenamento interno: 16 GB.
- Expansão: microSD de até 1 TB.
- Sistemas de arquivos informados: FAT32, exFAT e NTFS.
- Tela: LCD touch de 2,8 polegadas.
- Resolução informada: 480 × 640 pixels.
- Interface: totalmente sensível ao toque.

Essas especificações são informações fornecidas pelo usuário e ainda não foram verificadas por diagnóstico direto no dispositivo. A ABI efetivamente suportada, a densidade da tela, as características do firmware e o comportamento do gerenciamento de energia precisam ser confirmados durante o desenvolvimento.

## 3. Problema a resolver

O aplicativo Spotify convencional apresenta uma experiência de uso insatisfatória no hardware limitado e na tela pequena do player. O usuário encontrou o Sidetrack, que utiliza librespot, mas o projeto não funciona no Android 9 do dispositivo.

O trabalho consiste em adaptar o projeto existente em vez de criar do zero um cliente de streaming. Antes de implementar mudanças, deve-se inspecionar o checkout atual para determinar os motivos exatos da incompatibilidade e os ajustes necessários.

## 4. Objetivos principais

1. Tornar a aplicação compatível com Android 9 (API 28).
2. Permitir a instalação, inicialização e utilização no Oilsky M308.
3. Oferecer uma interface compacta, clara e adequada à tela touch de 480 × 640 pixels.
4. Garantir reprodução online estável.
5. Utilizar a melhor qualidade de streaming que a integração com o Spotify e a conta do usuário efetivamente disponibilizarem.
6. Reduzir consumo de memória, CPU, alocações e trabalho desnecessário na interface.
7. Manter a interface responsiva enquanto a música toca.
8. Preservar, sempre que viável, o núcleo de streaming librespot e a arquitetura atual do projeto.
9. Documentar build, instalação, testes, limitações e requisitos do ambiente de desenvolvimento.

## 5. Escopo funcional esperado

A experiência inicial deve priorizar:

- Conexão e sessão Spotify conforme o fluxo já suportado pelo projeto.
- Reprodução e pausa.
- Faixa anterior e próxima.
- Exibição do título, artista e capa do álbum.
- Progresso da faixa e busca temporal, quando suportada pela integração.
- Controle de volume, quando disponível.
- Acesso simples à biblioteca, busca, playlists, favoritos e fila, conforme as funcionalidades já existentes e a viabilidade na tela pequena.
- Tratamento claro de estados como conexão, carregamento, reprodução, pausa, erro de rede e sessão expirada.
- Continuidade da reprodução ao navegar entre telas e, se compatível com a implementação e com o Android 9, em segundo plano.

O foco inicial é reproduzir música online. Downloads, reprodução offline e cache persistente de faixas não fazem parte do escopo.

## 6. Requisitos de interface

A interface deve ser concebida para um dispositivo portátil de tela pequena, e não simplesmente reduzir proporcionalmente uma interface de celular.

Prioridades:

- Tela principal de reprodução com os controles essenciais imediatamente acessíveis.
- Botões grandes e espaçados, apropriados para interação por toque.
- Texto legível, com truncamento adequado para títulos e nomes longos.
- Navegação simples, com poucas camadas.
- Tema escuro, se compatível com a estrutura atual.
- Poucas animações e transições.
- Capas carregadas sob demanda, redimensionadas para a exibição e mantidas em cache limitado.
- Ausência de elementos essenciais cortados ou dependentes de rolagem horizontal.
- Tratamento adequado de densidade, insets e dimensões lógicas em dp.

A resolução física de 480 × 640 pixels não deve ser tratada como se fosse necessariamente 480 × 640 dp.

## 7. Requisitos de áudio

O objetivo é obter a melhor qualidade de streaming realmente disponível na implementação e na conta do Spotify, com reprodução contínua e estável.

A documentação do Sidetrack informa suporte a até 320 kbps em Ogg Vorbis. Essa informação deve ser conferida no código e na versão do projeto em uso. Ogg Vorbis a 320 kbps não é áudio lossless, e não se deve afirmar suporte a áudio sem perdas sem evidência verificável.

O trabalho deve avaliar o caminho de áudio existente, a ponte JNI, a saída de áudio Android, os buffers e o tratamento de falhas. Não deve implementar contorno de DRM, evasão de controles de acesso ou mecanismos para burlar limitações do Spotify.

## 8. Compatibilidade e restrições técnicas

- Android 9 corresponde à API 28.
- Alterar apenas `minSdk` não garante compatibilidade: APIs e dependências mais recentes podem falhar durante a execução.
- `minSdk`, `targetSdk` e `compileSdk` têm finalidades diferentes e devem ser configurados conscientemente.
- A arquitetura do processador não deve ser deduzida apenas pelo número de núcleos ou pela frequência comercial.
- Os targets Rust e as bibliotecas JNI devem corresponder às ABIs realmente suportadas pelo dispositivo.
- Versões de Gradle, Android Gradle Plugin, Kotlin, Compose, JDK e NDK precisam ser mutuamente compatíveis.
- Não se deve assumir que uma compilação bem-sucedida prova compatibilidade com o dispositivo real.

## 9. Critérios de sucesso

A adaptação deve ser considerada pronta para avaliação quando:

1. O APK instalar e iniciar no Android 9 do M308.
2. Os controles e textos essenciais couberem na tela e forem utilizáveis por toque.
3. O aplicativo conseguir estabelecer uma sessão válida e iniciar streaming online.
4. Reprodução, pausa, troca de faixa e demais controles implementados funcionarem.
5. O áudio permanecer estável durante um teste prolongado, sem falhas persistentes ou travamentos.
6. A interface continuar responsiva durante a reprodução.
7. O uso de memória e CPU for medido quando as ferramentas permitirem, e regressões forem identificadas.
8. O processo de compilação estiver documentado e não depender de segredos versionados.
9. Os testes realizados no emulador e no aparelho físico forem diferenciados claramente.
10. Qualquer requisito ainda não validado no Oilsky M308 estiver explicitamente identificado.

## 10. Limitações e riscos conhecidos

- O upstream pode depender de APIs incompatíveis com Android 9.
- Dependências atualizadas podem ter aumentado o requisito mínimo de Android.
- O firmware do fabricante pode impor limitações próprias.
- A integração com um serviço de terceiros pode mudar quando o Spotify atualiza seus sistemas.
- O comportamento depende de conectividade e de uma conta elegível.
- Clientes não oficiais podem estar sujeitos a alterações de compatibilidade, aos termos do serviço e a riscos para a conta. O projeto não deve afirmar ser afiliado, aprovado ou endossado pelo Spotify.
- A ABI, a densidade de tela e o comportamento real do aparelho ainda precisam ser confirmados.

## 11. Fora do escopo

- Criar um cliente Spotify inteiramente novo.
- Reimplementar o protocolo de streaming sem necessidade técnica demonstrada.
- Acrescentar downloads ou reprodução offline.
- Contornar DRM, controles de acesso ou restrições do serviço.
- Reescrever toda a arquitetura sem evidência de que isso é necessário.
- Adicionar funcionalidades que aumentem a complexidade sem melhorar a experiência essencial no M308.

## 12. Diretriz geral

O resultado desejado é um fork enxuto, confiável e fácil de manter, voltado ao Oilsky M308. Deve-se primeiro compreender o estado real do código, corrigir os bloqueadores de compatibilidade e só então adaptar a interface e otimizar o desempenho. A validação final deve ocorrer no próprio dispositivo sempre que possível.
