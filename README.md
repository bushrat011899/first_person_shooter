# Muskrats at Dawn

Muskrats at Dawn is a multiplayer first-person-shooter themed around 19th century warfare. The primary weapon for the game is a configurable musket. This musket has 4 parameters which control how it operates:

 * Barrel Length
    * Determines spread vs speed, and combustion completeness.
    * Negatively affects handling and weight.
 * Caliber
    * Determines the maximum bullet diameter.
    * Negatively affects handling, weight, and "slug point".
 * Barrel Thickness
    * Determines the maximum safe powder charge.
    * Negatively affects handling and weight.
 * Furniture
    * Determines handling.
    * Negatively affects weight.

A heavy musket will negatively affect player movement, encouraging players to chose a weapon which best suits their play style. For example, a conservative play style might favour a heavy musket for maximum range and damage. While an aggressive play style might favour minimal weight and maximum spread for up-close combat.

The musket itself requires 3 resources to fire:

 * Zero or More Bullets (integer)
 * Powder (float)
 * Wadding (boolean)

Wadding is required, but has no variable effect. Powder quantity determines the force imparted on the bullets. If no bullets are loaded, the gaseous blast is the only source of damage (which can still be quite effective!). Finally, the bullets are the projectiles which actually hit a target. A singular bullet will be most effective at longer ranges (e.g., a slug), while a collection of bullets may be more effective up close (e.g., buckshot). More bullet mass will require more powder to reach the same velocity.

A bullet can either be shot, garbage, or a slug depending on the caliber of the gun. If the bullet diameter is less than half of the caliber, it is shot. Between half and equal to, it is garbage, and equal to caliber it is a slug. Shot has medium range, medium damage, and medium spread. Slugs have maximum range, minimum damage, and minimum spread. Garbage has minimum range, maximum damage, and maximum spread.

# References

 * [Johan Helsing's `Matchbox`](https://github.com/johanhelsing/matchbox)
 * [Johan Helsing's Fork of `Bevy GGRS`](https://github.com/johanhelsing/bevy_ggrs/tree/bevy-0.10)