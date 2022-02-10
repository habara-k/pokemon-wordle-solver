let input = document.getElementById("input");
let output = document.getElementById("output");
let mode = document.getElementById("mode");
output.scrollTop = output.scrollHeight;
let json_obj = {};

input.addEventListener("keypress", on_enter);

function on_enter(e) {
  if (e.keyCode === 13) {
    if (json_obj === {}) {
      return;
    }

    if (!(input.value in json_obj.edges)) {
      output.value += "Incorrect input.\n";
      output.value += "Please restart, or type the correct response again.\n-> "
      output.scrollTop = output.scrollHeight;
      return;
    }
    json_obj = json_obj.edges[input.value];
    output.value += input.value + "\n";

    if ("guess" in json_obj) {
      output.value += json_obj.guess + "\n-> ";
    } else {
      output.value += "Congratulations!!!\n";
      output.value += "If you want to play again, please restart.";
    }
    input.value = "";
    output.scrollTop = output.scrollHeight;
  }
}

function restart() {
  output.value = "";
  input.value = "";
  json_obj = JSON.parse(tree_json[mode.value]);
  output.value += json_obj.guess + "\n-> ";
}

window.onload = function() {
  restart()
};
