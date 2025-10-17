export const computeSignals = async () => {
	await fetch('http://localhost:8080/api/run/signals');

	return;
};
