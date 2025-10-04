import cliConfetti from "cli-confetti";
import CliUpdate from "cli-update";

const SECONDS = 1000;

cliConfetti(
  {
    // The config
    chars: ["✨", "🍾", "🥂", "😃", "💯", "💙", "CONGRATULATIONS!"],
  },
  function (error, confetti) {
    if (error) throw error;
    // @ts-expect-error CluUpdate uses some kind of pre-ES6 class syntax that freaks out type inference
    CliUpdate.render(confetti);

    // Stop confetti after 5 seconds
    setTimeout(() => {
      // @ts-expect-error See above
      CliUpdate.render("🎉 Congratulations from QuickNode on building your first Solana program! 🎉");
      process.exit(0);
    }, 5 * SECONDS);
  },
);
