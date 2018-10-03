const socket = (() => {
    const socketAddr = (window.location.protocol == 'https:' && 'wss://' || 'ws://') + window.location.host + '/ws/';
    const socket = new WebSocket(socketAddr);

    //socket.onopen = event => { };
    
    const messages = document
	  .getElementById('messages');

    socket.onmessage = message => {
	update(message.data);
    };
    
    socket.onerror = e => {
	console.log(e);
    };

    socket.onclose = e => {
	console.log(e);
    };
    return socket;

    function update(message) {
	const deltaScroll = messages.scrollTopMax - messages.scrollTop;
	messages.innerHTML += message;
	if (deltaScroll < 10)
	    messages.scrollTop = messages.scrollTopMax;
    }
})();

{
    const sendButton = document
	  .getElementById('userinputsubmit');
    const messageField = document
	  .getElementById('userinputtext');
    
    sendButton.onclick = () => {
	const text = messageField.value;
	socket
	    .send(text);
	messageField.value = '';
	return false;
    };
    
    messageField.onkeyup = (key) => {
	if (key.keyCode === 13) // enter
	    sendButton.click();
    };
}
