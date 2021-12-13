# Trabalho 4
## Servidor
### O que foi feito:
- struct Request para armazenar requisições de clientes
- struct RequestLogEntry para armazenar uma entrada em uma lista de requisições
- struct ServerInfo para armazenar informações relevantes sobre o servidor
- recebe seu id e quantidade de servidores como argumentos do comando que cria o processo servidor
- inicializa cliente MQTT com id "inf1406/server-{id servidor}" para se comunicar com os processos cliente
- inicializa cliente MQTT com id "inf1406/server-monitor-{id servidor}" para se comunicar com o processo monitor
- cria thread para enviar heartbeat para o monitor a cada 5 segundos
- lê mensagens do tópico inf1406-reqs
- todos os servidores atualizam chave e valor ao receber requisição do tipo insere
- o servidor com id retornado pela função de hash envia o valor da consulta para o cliente
- identifica qual servidor deve substituir o que falhou

### Problemas encontrados:
- precisei criar 2 clientes MQTT, um para comunicação com o processo cliente e outro para o processo monitor
- servidor está recebendo mensagens do tópico inf1406-resp embora só esteja subscribed aos topico inf1406-reqs
- não consegui adicionar elementos do tipo RequestLogEntry para a lista com os logs de requisição. com isso também não consegui implementar a resposta pelo servidor substituto

## Testes realizados
- execução de funcionamento normal e com falha do servidor