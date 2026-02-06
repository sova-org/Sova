export function fuzzyScore(
	query: string,
	target: string,
	keywords?: string[]
): number {
	const q = query.toLowerCase();
	const t = target.toLowerCase();

	if (t === q) return 100;
	if (t.startsWith(q)) return 80;
	if (t.includes(q)) return 60;

	if (keywords?.some((k) => k.toLowerCase().includes(q))) return 50;

	let qi = 0;
	for (let ti = 0; ti < t.length && qi < q.length; ti++) {
		if (t[ti] === q[qi]) qi++;
	}
	if (qi === q.length) return 40;

	return 0;
}
