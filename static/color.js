(() => {
  const stringfy = color => `<div style="color:${color.code}"><input type="radio" name="usercolor" value="${color.code}" >${color.name}</input></div>`;

  const colors = [
    { name: "black",      code: "#000000" },
    { name: "white",      code: "#ffffff" },
    { name: "red",        code: "#ff0000" },
    { name: "orange",     code: "#ff8000" },
    { name: "yellow",     code: "#ffff00" },
    { name: "neon-green", code: "#00ff00" },
    { name: "neon-blue",  code: "#00ffff" },
    { name: "blue",       code: "#0000ff" },
    { name: "purple",     code: "#8000ff" },
    { name: "pink",       code: "#ff00ff" },
    { name: "dark-green", code: "#006600" },
  ];

  const radios = document
	.getElementById("colorradios");

  colors
    .forEach(color => radios.innerHTML += stringfy(color));

})();
