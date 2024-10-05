#!/usr/bin/env nix-shell
#! nix-shell -i perl -p perl538 -p perl538Packages.JSON -p perl538Packages.FileSlurp -p perl538Packages.DataDump -p perl538Packages.ModernPerl -p perl538Packages.TextSimpleTable

use Text::SimpleTable;
use Modern::Perl;
use feature qw(say postderef);
use File::Slurp;
use JSON;

my $json = JSON->new->utf8->pretty(1)->canonical(1)->allow_nonref;

my $file = '.github/workflows/main.yml';

my $x        = read_file('sources.json');
my $sources  = $json->decode($x);
my $f        = read_file($file);
my $workflow = $json->decode($f);

$workflow->{jobs}->{build}->{strategy}->{matrix}->{version}->@* =
  sort { $b cmp $a } map { $_->{package}->{name} } $sources->@*;

write_file( $file, $json->encode($workflow) );

sub to_number {
    my $v = shift;
    my $i = 100;
    my $n = 0;
    for my $d ($v =~ m/ (?<=[.-]|^) (\d+) (?=[.-]|$) /xgsm) {
        $n *= $i; $n += $d; $i **= 2;
    }
    if ($v =~ /-rc(\d+)/) {
        $n += 0.1 * $1;
    }
    return $n;
}

sub update_readme() {
    my $f = 'README.md';
    my $c = read_file($f);
    my @d = map { [ $_->{version} => $_ ] } sort { to_number($b->{version}) <=> to_number($a->{version}) } $sources->@*;
    my $m = "|Version|Package|Date|\n";
    $m.= "|---|---|---|\n";
    for my $k ( @d ) {
        $m.="|";
        $m.=  join "|", ($k->[0], "<b>" . $k->[1]->{package}->{name} . "</b>",
            $k->[1]->{date} );
        $m.="|\n";
    }
    $c =~ s{
     (?<=\<\!--START--\>\R)
      .*
     (?=\<\!--END--\>\R)
    }{$m}xgms;

    write_file( $f, $c );
}

update_readme();
