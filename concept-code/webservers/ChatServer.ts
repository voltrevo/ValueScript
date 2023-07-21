export default class ChatServer {
  messages: string[] = [];

  post(message: string) {
    this.messages.push(message);
  }

  recent() {
    return this.messages.slice(-10);
  }
}
