<script setup>
// Illustrative only — the shape of a real Plan, with made-up numbers.
const plan = [
  {
    seq: 1,
    by: '05 Sep',
    from: 'Salary account',
    to: 'Spends account',
    amount: '₹1,04,000',
    reason: 'fund card dues + fixed expenses',
    kind: 'transfer',
  },
  {
    seq: 2,
    by: '08 Sep',
    from: 'Spends account',
    to: 'HDFC credit card',
    amount: '₹64,310',
    reason: 'statement due 10 Sep',
    kind: 'card',
  },
  {
    seq: 3,
    by: '12 Sep',
    from: 'Spends account',
    to: 'ICICI credit card',
    amount: '₹18,940',
    reason: 'statement due 14 Sep',
    kind: 'card',
  },
  {
    seq: 4,
    by: '15 Sep',
    from: 'Salary account',
    to: 'Investment account',
    amount: '₹85,000',
    reason: 'surplus after buffer of ₹2,00,000',
    kind: 'transfer',
  },
  {
    seq: 5,
    by: '15 Sep',
    from: 'Investment account',
    to: 'Nifty 50 index',
    amount: '₹45,000',
    reason: 'underweight 2.1%',
    kind: 'invest',
  },
  {
    seq: 6,
    by: '15 Sep',
    from: 'Investment account',
    to: 'Gold',
    amount: '₹25,000',
    reason: 'underweight 1.4%',
    kind: 'invest',
  },
  {
    seq: 7,
    by: '15 Sep',
    from: 'Investment account',
    to: 'US equity (IBKR)',
    amount: '₹15,000',
    reason: 'LRS used ₹4.2L of ₹10L this FY',
    kind: 'invest',
  },
]

const dot = {
  transfer: 'bg-muted-foreground',
  card: 'bg-primary',
  invest: 'bg-positive',
}
</script>

<template>
  <div class="overflow-hidden rounded-xl border bg-card shadow-sm">
    <div class="flex items-center justify-between border-b px-5 py-3.5">
      <div class="flex items-center gap-2.5">
        <span class="h-2 w-2 rounded-full bg-positive" />
        <span class="text-sm font-semibold">Plan for September</span>
      </div>
      <span class="font-mono text-xs text-muted-foreground">7 steps</span>
    </div>

    <!-- Wide content scrolls inside its own container rather than the page -->
    <div class="overflow-x-auto">
      <table class="w-full min-w-[44rem] text-sm">
        <tbody>
          <tr v-for="s in plan" :key="s.seq" class="border-b last:border-0">
            <td class="w-10 py-3.5 pl-5 font-mono text-xs text-muted-foreground">
              {{ s.seq }}
            </td>
            <td class="w-24 whitespace-nowrap py-3.5 font-mono text-xs text-muted-foreground">
              by {{ s.by }}
            </td>
            <td class="py-3.5">
              <div class="flex items-center gap-2.5">
                <span class="h-1.5 w-1.5 shrink-0 rounded-full" :class="dot[s.kind]" />
                <span class="whitespace-nowrap">{{ s.from }}</span>
                <span class="text-muted-foreground">&rarr;</span>
                <span class="whitespace-nowrap font-medium">{{ s.to }}</span>
              </div>
            </td>
            <td class="whitespace-nowrap py-3.5 text-right font-mono font-medium tabular-nums">
              {{ s.amount }}
            </td>
            <td class="whitespace-nowrap py-3.5 pl-6 pr-5 text-right text-xs text-muted-foreground">
              {{ s.reason }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <p class="border-t bg-muted/40 px-5 py-3 text-xs text-muted-foreground">
      Illustrative. Real plans are computed from your own statements and account policies.
    </p>
  </div>
</template>
