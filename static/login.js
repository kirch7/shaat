const repassword = document
      .getElementById("repassword");
const password0 = document
      .getElementById("password0");
repassword.hidden = true;
password0.hidden = true;

const submitButton = document
      .getElementById("submitButton");
submitButton.hidden = true;

const colors = document
      .getElementById("colorradios");
colors.hidden = true;

const checkusernameEvent = e => {
    let u = document
	.getElementById("username")
	.value;
    let query = '{user(username: ' + `"${u}"` + ') { username }}';
    fetch('/graphql',
	  {
	      method: 'post',
	      headers: {
		  'Accept': 'application/json',
		  'Content-Type': 'application/json',
	      },
	      body: JSON.stringify({ query })
	  }
	 )
	.then(response => response.json())
	.then(json => {
	    if (u.length < 2) {
		hidePasswordEvent();
	    } else if (json.errors) {
		password0.hidden = false;
		repassword.hidden = false;
		colors.hidden = false;
		submitButton.hidden = false;
		submitButton.value = "Registra";
	    } else {
		password0.hidden = false;
		repassword.hidden = true;
		colors.hidden = true;
		submitButton.hidden = false;
		submitButton.value = "Entra";
	    }
	});
};

const hidePasswordEvent = e => {
    password0.hidden = true;
    repassword.hidden = true;
    colors.hidden = true;
    submitButton.hidden = true;
};

document
    .getElementById("checkusername")
    .onclick = checkusernameEvent;

const username = document
      .getElementById("username");
username.onchange = hidePasswordEvent;
username.onkeyup = e => {
    if (e.keyCode === 13) // enter
     	checkusernameEvent(e);
    else 
	hidePasswordEvent(e);
};

const authform = document
      .getElementById("authform");
authform.onkeydown = e => {
    if (e.keyCode === 13)
	e.preventDefault();
};

