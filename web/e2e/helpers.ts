/** Helper to select a value in a shadcn-vue Select component by its trigger locator. */
export async function selectOption(page: any, trigger: any, optionName: string) {
  await trigger.click()
  await page.getByRole('option', { name: optionName, exact: true }).click()
}
